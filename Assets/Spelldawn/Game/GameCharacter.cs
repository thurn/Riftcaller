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
using System.Collections.Generic;
using System.Linq;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using Spelldawn.World;
using TMPro;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class GameCharacter : StackObjectDisplay, ArrowService.IArrowDelegate
  {
    [Header("Character")]
    [SerializeField] Registry _registry = null!;
    [SerializeField] TextMeshPro _scoreText = null!;
    [SerializeField] PlayerName _owner;
    [SerializeField] GameObject _speechBubble = null!;
    [SerializeField] TextMeshPro _speechBubbleText = null!;
    [SerializeField] AnimatedCharacter _character = null!;
    ISet<RoomIdentifier>? _validRoomsToVisit;

    public PlayerSide Side { get; set; }

    protected override Registry Registry => _registry;

    protected override GameContext DefaultGameContext() => GameContext.GameCharacter;

    protected override float? CalculateObjectScale(int index, int count) => 0f;

    public override float DefaultScale => 2.0f;

    public override bool IsContainer() => false;

    protected override void OnStart()
    {
      _character.SetSpeed(0f);
    }

    public void OnRaidStateChanged(bool raidActive)
    {
    }

    public void SetFacingDirection(GameCharacterFacingDirection direction)
    {
      switch (direction)
      {
        case GameCharacterFacingDirection.Up:
          _character.SetDirection(AnimatedCharacter.Direction.Up);
          break;
        case GameCharacterFacingDirection.Down:
          _character.SetDirection(AnimatedCharacter.Direction.Down);          
          break;
        case GameCharacterFacingDirection.Left:
          _character.SetDirection(AnimatedCharacter.Direction.Left);          
          break;
        case GameCharacterFacingDirection.Right:
          _character.SetDirection(AnimatedCharacter.Direction.Right);          
          break;
        case GameCharacterFacingDirection.Unspecified:
        default:
          throw new ArgumentOutOfRangeException(nameof(direction), direction, null);
      }
    }

    public void DisableAnimation()
    {
    }

    public void RenderPlayerInfo(PlayerInfo playerInfo)
    {
      _validRoomsToVisit = playerInfo.ValidRoomsToVisit.ToHashSet();
    }

    public void RenderScore(ScoreView scoreView)
    {
      _scoreText.text = scoreView.Score.ToString();
    }

    public void DisplaySpeechBubble(string text)
    {
      _speechBubble.SetActive(true);
      _speechBubbleText.text = text;
    }

    public void HideSpeechBubble()
    {
      _speechBubble.SetActive(false);
      _speechBubbleText.text = "";      
    }

    public override bool CanHandleMouseDown() => !Registry.CapabilityService.AnyOverlayOpen();

    public override void MouseDown()
    {
      base.MouseDown();

      if (_owner == PlayerName.User && _registry.CapabilityService.CanInitiateAction())
      {
        _registry.StaticAssets.PlayCardSound();
        switch (Side)
        {
          case PlayerSide.Champion:
            _registry.ArrowService.ShowArrow(ArrowService.Type.Red, transform, this);
            break;
          case PlayerSide.Overlord:
            _registry.ArrowService.ShowArrow(ArrowService.Type.Green, transform, this);
            break;
        }
      }
    }

    protected override void LongPress()
    {
      StartCoroutine(_registry.LongPressCardBrowser.BrowseCards(this));
    }

    public void OnArrowMoved(Vector3 position)
    {
      if (_owner == PlayerName.User)
      {
        _registry.ArenaService.ShowRoomSelectorForMousePosition(Errors.CheckNotNull(_validRoomsToVisit));
      }
    }

    public void OnArrowReleased(Vector3 position)
    {
      if (_registry.ArenaService.CurrentRoomSelector is { } selectedRoom)
      {
        switch (Side)
        {
          case PlayerSide.Champion:
            _registry.ActionService.HandleAction(new ClientAction
            {
              InitiateRaid = new InitiateRaidAction
              {
                RoomId = selectedRoom.RoomId
              }
            });
            break;
          case PlayerSide.Overlord:
            _registry.ActionService.HandleAction(new ClientAction
            {
              LevelUpRoom = new LevelUpRoomAction
              {
                RoomId = selectedRoom.RoomId
              }
            });
            break;
        }
      }
      else
      {
        _registry.StaticAssets.PlayCardSound();
      }

      _registry.ArenaService.HideRoomSelector();
    }

    protected override void OnSetGameContext(GameContext _, GameContext gameContext)
    {
    }
  }
}