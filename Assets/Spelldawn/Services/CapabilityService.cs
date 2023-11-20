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

#nullable enable

using Spelldawn.Game;
using Spelldawn.Protos;
using UnityEngine;

namespace Spelldawn.Services
{
  public sealed class CapabilityService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;

    [SerializeField] PlayerName _currentPriority;

    public PlayerName CurrentPriority
    {
      get => _currentPriority;
      set => _currentPriority = value;
    }

    /// <summary>
    /// Can the user *start* performing a character arrow action such as dragging a raid arrow.
    /// </summary>
    /// 
    /// This is allowed more leniently than actually *performing* an action as defined by
    /// <see cref="CanExecuteAction"/> below.
    /// 
    public bool CanDragCharacterArrow() => !_registry.CardService.CurrentlyDragging &&
                                       !AnyOverlayOpen() &&
                                       !_registry.DocumentService.IsAnyPanelOpen();
    
    /// <summary>
    ///  Can the user currently play cards that are otherwise legal to play according to the game rules? 
    /// </summary>
    public bool CanMoveCards() => !_registry.CardService.CurrentlyDragging && 
                                  !_registry.LongPressOverlay.Enabled &&
                                  !_registry.DocumentService.IsAnyPanelOpen();

    public bool AnyOverlayOpen() => _registry.RaidOverlay.Enabled ||
                                    _registry.InterfaceOverlay.Enabled ||
                                    _registry.LongPressOverlay.Enabled;

    /// <summary>
    /// Can the user currently zoom a card that exists in the provided GameContext.
    /// </summary>
    public bool CanInfoZoom(Displayable displayable, GameContext gameContext)
    {
      if (_registry.DocumentService.IsAnyPanelOpen())
      {
        return false;
      }

      switch (gameContext)
      {
        case GameContext.ArenaRaidParticipant:
        case GameContext.RaidParticipant:
          // If a card is a top-level raid participant, it can be info zoomed. However if a card is *part* of
          // a parent display that is participating in a raid (e.g. it is part of the discard pile that is 
          // being targeted), then it cannot be info zoomed and the long-press browser is used instead.
          return displayable.Parent == _registry.RaidService.RaidParticipants ||
                 displayable.Parent == _registry.ArenaService.LeftItems ||
                 displayable.Parent == _registry.ArenaService.RightIems;
        case GameContext.Browser:
        case GameContext.RewardBrowser:
        case GameContext.LongPressBrowser:
        case GameContext.RevealedCardsBrowser:
        case GameContext.Hand:
          return true;
        case GameContext.Deck:
        case GameContext.DiscardPile:
        case GameContext.BehindArena:
          return false;
        default:
          return !AnyOverlayOpen();
      }
    }

    /// <summary>
    /// Can the user currently perform a game action of the provided type?
    /// </summary>
    public bool CanExecuteAction(ClientAction.ActionOneofCase actionType) => actionType switch
    {
      ClientAction.ActionOneofCase.StandardAction => CanAct(
        allowInOverlay: true,
        actionPointRequired: false,
        allowWithPanelOpen: true),
      ClientAction.ActionOneofCase.FetchPanel => true,
      ClientAction.ActionOneofCase.GainMana => CanAct(),
      ClientAction.ActionOneofCase.DrawCard => CanAct(),
      ClientAction.ActionOneofCase.PlayCard => CanMoveCards(),
      ClientAction.ActionOneofCase.ProgressRoom => CanAct(),
      ClientAction.ActionOneofCase.InitiateRaid => CanAct(),
      ClientAction.ActionOneofCase.MoveCard => CanMoveCards(),
      _ => false
    };

    bool CanAct(bool allowInOverlay = false, bool actionPointRequired = true, bool allowWithPanelOpen = false) =>
      !_registry.CardService.CurrentlyDragging &&
      (allowWithPanelOpen || !_registry.DocumentService.IsAnyPanelOpen()) &&
      (allowInOverlay || !AnyOverlayOpen()) &&
      (allowInOverlay || !_registry.RaidService.RaidActive) &&
      (!actionPointRequired || _registry.ActionDisplayForPlayer(PlayerName.User).AvailableActions > 0);

    public bool CanDragInterfaceElement() => true;

    void Update()
    {
      var userLight = _registry.ActiveLightForPlayer(PlayerName.User);
      var opponentLight = _registry.ActiveLightForPlayer(PlayerName.Opponent);

      if (userLight == null || opponentLight == null)
      {
        return;
      }

      switch (_registry.CapabilityService.CurrentPriority)
      {
        case PlayerName.User when CanExecuteAction(ClientAction.ActionOneofCase.PlayCard):
          userLight.SetActive(true);
          opponentLight.SetActive(false);
          break;
        case PlayerName.Opponent:
          opponentLight.SetActive(true);
          userLight.SetActive(false);
          break;
        case PlayerName.Unspecified:
        default:
          userLight.SetActive(false);
          opponentLight.SetActive(false);
          break;
      }
    }
  }
}