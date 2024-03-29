// Copyright © Riftcaller 2021-present

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
using Riftcaller.Assets;
using Riftcaller.Game;
using Riftcaller.Protos;
using Riftcaller.Utils;
using UnityEngine;
using UnityEngine.AddressableAssets;
using UnityEngine.SceneManagement;

#nullable enable

namespace Riftcaller.Services
{
  public sealed class ArenaService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] LinearObjectDisplay _leftItems = null!;
    [SerializeField] LinearObjectDisplay _rightItems = null!;
    [SerializeField] List<Room> _rooms = null!;
    [SerializeField] AssetReferenceGameObject _initiateRaidPrefabReference = null!;
    [SerializeField] AssetReferenceGameObject _levelUpRoomPrefabReference = null!;
    [SerializeField] Room? _curentRoomSelector;
    [SerializeField] AssetReference _arenaScene = null!;
    bool _arenaSceneLoaded;

    public Room? CurrentRoomSelector => _curentRoomSelector;
    public ObjectDisplay LeftItems => _leftItems;
    public ObjectDisplay RightIems => _rightItems;
    public bool RoomsOnBottom { get; private set; }

    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];

    public IEnumerator Initialize()
    {
      if (UseProductionAssets.ShouldUseProductionAssets)
      {
        yield return Addressables.LoadSceneAsync(_arenaScene, LoadSceneMode.Additive);
        _arenaSceneLoaded = true;
        FlipView();
        foreach (var target in GameObject.FindGameObjectsWithTag("RemoveOnLoad"))
        {
          Destroy(target);
        }
      }
    }

    public void UpdateViewForSide(PlayerSide side)
    {
      RoomsOnBottom = side == PlayerSide.Covenant;
      FlipView();
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
          LogUtils.LogError($"Unknown item location: {position.ItemLocation}");
          return _rightItems.AddObject(card, animate);
      }
    }

    /// <summary>
    /// Shows a green highlight on the requested room. Returns true if this room is on the right side of the screen.
    /// </summary>
    public bool ShowRoomSelectorForRoom(RoomIdentifier roomId)
    {
      HideRoomSelector();
      var room = FindRoom(roomId);
      room.SpriteRenderer.enabled = true;
      _curentRoomSelector = room;
      return _registry.MainCamera.WorldToScreenPoint(room.transform.position).x > Screen.width / 2.0;      
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
      var character = _registry.GameCharacterForPlayer(command.Initiator).transform;
      
      yield return TweenUtils.Sequence("RoomVisit")
        .Append(character
          .DOMove(room.position, 0.3f).SetEase(Ease.OutSine))
        .AppendCallback(() =>
        {
          StartCoroutine(_registry.AssetPoolService.CreateFromReference(command.VisitType switch
          {
            RoomVisitType.InitiateRaid => _initiateRaidPrefabReference,
            RoomVisitType.ProgressRoom => _levelUpRoomPrefabReference,
            _ => throw new ArgumentOutOfRangeException(nameof(command.VisitType), command.VisitType, null)
          }, room.position, onCreate: result => result.transform.localScale = 5f * Vector3.one));
        })
        .Append(character
          .DOMove(_registry.CharacterPositionForPlayer(command.Initiator).transform.position, 0.3f)
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

    void FlipView()
    {
      if (!_arenaSceneLoaded)
      {
        return;
      }
      
      // The main ArenaService has a SceneBackground object, as does the loaded visual scene, so we find
      // them all here. There's probably a less hacky way to do this. 
      var backgrounds = FindObjectsOfType<SceneBackground>();
      if (backgrounds.Length != 2)
      {
        // If this is failing, check whether the target scene is correctly in an asset bundle. It does not
        // need to be in the "build settings" panel
        LogUtils.LogError($"Expected two SceneBackground objects but found {backgrounds.Length}");
      }

      foreach (var background in backgrounds)
      {
        background.SetRoomsOnBottom(RoomsOnBottom);
      }      
    }

    void SetGameObjectsEnabledForPlayer(PlayerName playerName, SetGameObjectsEnabledCommand command)
    {
      _registry.ActionDisplayForPlayer(playerName).gameObject.SetActive(command.GameObjectsEnabled);
      _registry.ManaDisplayForPlayer(playerName).gameObject.SetActive(command.GameObjectsEnabled);
      _registry.GameCharacterForPlayer(playerName).gameObject.SetActive(command.GameObjectsEnabled);
      SetObjectDisplayActive(_registry.DeckForPlayer(playerName), command);
      SetObjectDisplayActive(_registry.DiscardPileForPlayer(playerName), command);
      SetObjectDisplayActive(_registry.GameCharacterForPlayer(playerName), command);
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