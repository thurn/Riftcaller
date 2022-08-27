// Copyright © Spelldawn 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

using System;
using System.Collections;
using System.Collections.Generic;
using DG.Tweening;
using Spelldawn.Assets;
using Spelldawn.Game;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEditor;
using UnityEngine;
using UnityEngine.AddressableAssets;
using UnityEngine.SceneManagement;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class ArenaService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] LinearObjectDisplay _leftItems = null!;
    [SerializeField] LinearObjectDisplay _rightItems = null!;
    [SerializeField] List<Room> _rooms = null!;
    [SerializeField] SceneBackground _sceneBackground = null!;
    [SerializeField] TimedEffect _initiateRaidPrefab = null!;
    [SerializeField] AssetReferenceGameObject _initiateRaidPrefabReference = null!;
    [SerializeField] TimedEffect _levelUpRoomPrefab = null!;
    [SerializeField] AssetReferenceGameObject _levelUpRoomPrefabReference = null!;
    [SerializeField] Room? _curentRoomSelector;
    [SerializeField] AssetReference _arenaScene = null!;

    public Room? CurrentRoomSelector => _curentRoomSelector;

    public ObjectDisplay LeftItems => _leftItems;
    public ObjectDisplay RightIems => _rightItems;

    public bool RoomsOnBottom { get; private set; }

    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];

    public IEnumerator Initialize()
    {
      if (AssetPreference.UseProductionAssets)
      {
        yield return Addressables.LoadSceneAsync(_arenaScene, LoadSceneMode.Additive);
        foreach (var target in GameObject.FindGameObjectsWithTag("RemoveOnLoad"))
        {
          Destroy(target);
        }
      }
    }

    public void UpdateViewForSide(PlayerSide side)
    {
      RoomsOnBottom = side == PlayerSide.Overlord;
      _sceneBackground.SetRoomsOnBottom(side == PlayerSide.Overlord);
    }

    public Room FindRoom(RoomIdentifier roomId)
    {
      var result = _rooms.Find(r => r.RoomId == roomId);
      return Errors.CheckNotNull(result);
    }

    public ObjectDisplay ObjectDisplayForLocation(ClientItemLocation location) => location switch
    {
      ClientItemLocation.Left => _leftItems,
      ClientItemLocation.Right => _rightItems,
      _ => throw new ArgumentOutOfRangeException(nameof(location), location, null)
    };

    public IEnumerator AddAsItem(Displayable card, ObjectPositionItem position, bool animate)
    {
      switch (position.ItemLocation)
      {
        case ClientItemLocation.Left:
          return _leftItems.AddObject(card, animate);
        case ClientItemLocation.Right:
          return _rightItems.AddObject(card, animate);
        default:
          Debug.LogError($"Unknown item location: {position.ItemLocation}");
          return _rightItems.AddObject(card, animate);
      }
    }

    public void ShowRoomSelectorForMousePosition(ISet<RoomIdentifier> validRooms)
    {
      HideRoomSelector();

      var ray = _registry.MainCamera.ScreenPointToRay(Input.mousePosition);
      var hits = Physics.RaycastNonAlloc(ray, _raycastHitsTempBuffer, 100);

      for (var i = 0; i < hits; ++i)
      {
        var hit = _raycastHitsTempBuffer[i];
        var selector = hit.collider.GetComponent<Room>();
        if (selector)
        {
          if (validRooms.Contains(selector.RoomId))
          {
            selector.SpriteRenderer.enabled = true;
            _curentRoomSelector = selector;
            break;
          }
        }
      }

      Array.Clear(_raycastHitsTempBuffer, 0, _raycastHitsTempBuffer.Length);
    }

    public void HideRoomSelector()
    {
      if (_curentRoomSelector)
      {
        _curentRoomSelector!.SpriteRenderer.enabled = false;
        _curentRoomSelector = null;
      }
    }

    public IEnumerator HandleVisitRoom(VisitRoomCommand command)
    {
      var room = FindRoom(command.RoomId).transform;
      var identityCard = _registry.IdentityCardForPlayer(command.Initiator).transform;
      yield return TweenUtils.Sequence("RoomVisit")
        .Append(identityCard
          .DOMove(room.position, 0.3f).SetEase(Ease.OutSine))
        .AppendCallback(() =>
        {
          StartCoroutine(_registry.AssetPoolService.CreateFromReference(command.VisitType switch
          {
            RoomVisitType.InitiateRaid => _initiateRaidPrefabReference,
            RoomVisitType.LevelUpRoom => _levelUpRoomPrefabReference,
            _ => throw new ArgumentOutOfRangeException(nameof(command.VisitType), command.VisitType, null)
          }, room.position, onCreate: result => result.transform.localScale = 5f * Vector3.one));
        })
        .Append(identityCard
          .DOMove(_registry.IdentityCardPositionForPlayer(command.Initiator).transform.position, 0.3f)
          .SetEase(Ease.OutSine))
        .WaitForCompletion();
    }

    public IEnumerator HandleSetGameObjectsEnabled(SetGameObjectsEnabledCommand command)
    {
      foreach (var room in _rooms)
      {
        SetObjectDisplayActive(room.FrontCards, command);
        SetObjectDisplayActive(room.BackCards, command);
      }

      SetObjectDisplayActive(_leftItems, command);
      SetObjectDisplayActive(_rightItems, command);

      SetGameObjectsEnabledForPlayer(PlayerName.User, command);
      SetGameObjectsEnabledForPlayer(PlayerName.Opponent, command);
      yield break;
    }

    void SetGameObjectsEnabledForPlayer(PlayerName playerName, SetGameObjectsEnabledCommand command)
    {
      _registry.ActionDisplayForPlayer(playerName).gameObject.SetActive(command.GameObjectsEnabled);
      _registry.ManaDisplayForPlayer(playerName).gameObject.SetActive(command.GameObjectsEnabled);
      _registry.IdentityCardForPlayer(playerName).gameObject.SetActive(command.GameObjectsEnabled);
      SetObjectDisplayActive(_registry.DeckForPlayer(playerName), command);
      SetObjectDisplayActive(_registry.DiscardPileForPlayer(playerName), command);
      SetObjectDisplayActive(_registry.IdentityCardForPlayer(playerName), command);
      SetObjectDisplayActive(_registry.HandForPlayer(playerName), command);
    }

    void SetObjectDisplayActive(ObjectDisplay objectDisplay, SetGameObjectsEnabledCommand command)
    {
      foreach (var child in objectDisplay.AllObjects)
      {
        child.gameObject.SetActive(command.GameObjectsEnabled);
      }

      objectDisplay.gameObject.SetActive(command.GameObjectsEnabled);
    }
  }
}