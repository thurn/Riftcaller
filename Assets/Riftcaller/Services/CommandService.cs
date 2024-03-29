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
using System.Linq;
using Riftcaller.Assets;
using Riftcaller.Common;
using Riftcaller.Game;
using Riftcaller.Masonry;
using Riftcaller.Protos;
using Riftcaller.Utils;
using UnityEngine;
using UnityEngine.SceneManagement;

#nullable enable

namespace Riftcaller.Services
{
  public sealed class CommandService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] bool _currentlyHandling;
    
    readonly Queue<CommandList> _queue = new();
    public bool Active => _currentlyHandling;
    public ClientMetadata ClientMetadata { get; private set; } = new();

    public IEnumerator HandleCommands(IEnumerable<GameCommand> commands)
    {
      var list = new CommandList();
      list.Commands.AddRange(commands);
      return HandleCommands(list);
    }

    public IEnumerator HandleCommands(params GameCommand[] commands)
    {
      return HandleCommands(commands.ToList());
    }

    public IEnumerator HandleCommands(CommandList commandList)
    {
      _queue.Enqueue(commandList);
      yield return new WaitUntil(() => _currentlyHandling == false && _queue.Count == 0);
    }

    void Update()
    {
      if (_queue.Count > 0 && !_currentlyHandling)
      {
        _currentlyHandling = true;
        StartCoroutine(HandleCommandsAsync(_queue.Dequeue(), () => { _currentlyHandling = false; }));
      }
    }
    
    IEnumerator HandleCommandsAsync(CommandList commandList, Action? onComplete = null)
    {
      SetMetadata(commandList.Metadata);
      _registry.AnalyticsService.SetMetadata(commandList.LoggingMetadata);

      if (commandList.Commands.Count == 0)
      {
        onComplete?.Invoke();
        yield break;
      }
      
      if (commandList.Commands.Any(c => c.CommandCase == GameCommand.CommandOneofCase.UpdateGameView))
      {
        // Clear UI during animations
        _registry.DocumentService.ClearMainControls();
      }

      yield return _registry.AssetService.LoadAssets(commandList);

      if (commandList.Commands.Any(c => c.CommandCase != GameCommand.CommandOneofCase.InfoZoom))
      {
        var names = string.Join(",", commandList.Commands.Select(c => c.CommandCase));
        LogUtils.Log($"Handling Commands: {names}");        
      }
      
      foreach (var command in commandList.Commands)
      {
        switch (command.CommandCase)
        {
          case GameCommand.CommandOneofCase.Debug:
            HandleClientDebugCommand(command.Debug);
            break;
          case GameCommand.CommandOneofCase.UpdatePanels:
            _registry.DocumentService.HandleUpdatePanels(command.UpdatePanels);
            break;
          case GameCommand.CommandOneofCase.TogglePanel: 
            _registry.DocumentService.TogglePanel(command.TogglePanel);
            break;
          case GameCommand.CommandOneofCase.Delay:
            yield return new WaitForSeconds(DataUtils.ToSeconds(command.Delay.Duration, 0));
            break;
          case GameCommand.CommandOneofCase.UpdateGameView:
            yield return HandleUpdateGameView(command.UpdateGameView.Game, command.UpdateGameView.Animate);
            break;
          case GameCommand.CommandOneofCase.VisitRoom:
            yield return _registry.ArenaService.HandleVisitRoom(command.VisitRoom);
            break;
          case GameCommand.CommandOneofCase.PlaySound:
            AssetUtil.PlayOneShot(_registry.MainAudioSource,
              _registry.AssetService.GetAudioClip(command.PlaySound.Sound));
            break;
          case GameCommand.CommandOneofCase.SetMusic:
            _registry.MusicService.SetMusicState(command.SetMusic.MusicState);
            break;
          case GameCommand.CommandOneofCase.FireProjectile:
            yield return
              _registry.ObjectPositionService.HandleFireProjectileCommand(command.FireProjectile);
            break;
          case GameCommand.CommandOneofCase.PlayEffect:
            yield return HandlePlayEffect(command.PlayEffect);
            break;
          case GameCommand.CommandOneofCase.DisplayGameMessage:
            yield return _registry.GameMessage.Show(command.DisplayGameMessage);
            break;
          case GameCommand.CommandOneofCase.SetGameObjectsEnabled:
            yield return _registry.ArenaService.HandleSetGameObjectsEnabled(command.SetGameObjectsEnabled);
            break;
          case GameCommand.CommandOneofCase.DisplayRewards:
            yield return _registry.RewardDisplay.HandleDisplayRewards(command.DisplayRewards);
            break;
          case GameCommand.CommandOneofCase.LoadScene:
            yield return HandleLoadScene(command.LoadScene);
            break;
          case GameCommand.CommandOneofCase.MoveGameObjects:
            yield return _registry.ObjectPositionService.HandleMoveGameObjectsCommand(
              command.MoveGameObjects);
            break;
          case GameCommand.CommandOneofCase.SetCardMovementEffect:
            _registry.CardService.HandleSetCardMovementEffect(command.SetCardMovementEffect);
            break;
          case GameCommand.CommandOneofCase.CreateTokenCard:
            yield return HandleCreateTokenCard(command.CreateTokenCard);
            break;
          case GameCommand.CommandOneofCase.UpdateWorldMap:
            yield return _registry.WorldMap.HandleUpdateWorldMap(command.UpdateWorldMap);
            break;
          case GameCommand.CommandOneofCase.RenderScreenOverlay:
            _registry.DocumentService.SetScreenOverlay(command.RenderScreenOverlay.Node);
            break;
          case GameCommand.CommandOneofCase.UpdateInterface:
            yield return _registry.UpdateInterfaceService.HandleUpdate(command.UpdateInterface);
            break;
          case GameCommand.CommandOneofCase.Conditional:
            yield return HandleConditionalCommand(command.Conditional);
            break;
          case GameCommand.CommandOneofCase.InfoZoom:
            _registry.CardService.HandleInfoZoomCommand(command.InfoZoom);
            break;
          case GameCommand.CommandOneofCase.SetKeyboardShortcuts:
            _registry.InputService.SetKeyboardShortcuts(command.SetKeyboardShortcuts.MappingList);
            break;
          case GameCommand.CommandOneofCase.TurnFaceDownArenaAnimation:
            yield return _registry.CardService.HandleTurnFaceDownArenaAnimation(command.TurnFaceDownArenaAnimation);
            break;
          case GameCommand.CommandOneofCase.None:
            break;
          default:
            LogUtils.LogError($"Unknown game command: {command.CommandCase}");
            break;
        }
      }

      _registry.CardService.OnCommandsFinished();
      onComplete?.Invoke();
    }

    void SetMetadata(ClientMetadata? metadata)
    {
      if (metadata != null)
      {
        if (!ClientMetadata.Equals(metadata))
        {
          var logs = new List<string>();
          if (!string.IsNullOrEmpty(metadata.AdventureId))
          {
            logs.Add($"Adventure: {metadata.AdventureId}");
          }
          if (!string.IsNullOrEmpty(metadata.GameId))
          {
            logs.Add($"Game: {metadata.GameId}");
          }          
          LogUtils.Log(string.Join(" ", logs));
        }

        ClientMetadata = metadata;
      }
    }
    
    IEnumerator HandleConditionalCommand(ConditionalCommand command)
    {
      var query = command.Query.QueryCase switch
      {
        ConditionalQuery.QueryOneofCase.ElementExists => 
          _registry.UpdateInterfaceService.ElementExists(command.Query.ElementExists),
        _ => false
      };

      if (query)
      {
        yield return HandleCommandsAsync(command.IfTrue);
      }
      else
      {
        yield return HandleCommandsAsync(command.IfFalse);
      }
    }

    IEnumerator HandlePlayEffect(PlayEffectCommand command)
    {
      var positionTransform = command.Position.EffectPositionCase switch
      {
        PlayEffectPosition.EffectPositionOneofCase.GameObject =>
          _registry.ObjectPositionService.Find(command.Position.GameObject).transform,
        _ => throw new ArgumentOutOfRangeException()
      };
      var anchor = positionTransform.Find("EffectAnchor");
      var position = anchor ? anchor.position : positionTransform.position;

      var effect = _registry.AssetPoolService.Create(_registry.AssetService.GetEffect(command.Effect), position);

      effect.SetGameContext(command.ArenaEffect ? GameContext.Arena : GameContext.Effects);
      if (command.StartColor != null)
      {
        effect.SetStartColor(Mason.ToUnityColor(command.StartColor));
      }

      if (anchor)
      {
        effect.transform.forward = anchor.forward;           
      }
      else
      {
        var rotation = Quaternion.LookRotation(position - _registry.MainCamera.transform.position);
        effect.transform.rotation = rotation;        
      }
      
      if (command.Scale is { } scale)
      {
        effect.transform.localScale = scale * Vector3.one;
      }

      if (command.Sound != null)
      {
        AssetUtil.PlayOneShot(_registry.MainAudioSource, _registry.AssetService.GetAudioClip(command.Sound));
      }
      

      yield return new WaitForSeconds(DataUtils.ToSeconds(command.Duration, 0));
    }

    IEnumerator HandleUpdateGameView(GameView game, bool animate)
    {
      _registry.CardService.SetDeckViews(game.User?.DeckView, game.Opponent?.DeckView);

      if (game.User != null)
      {
        if (game.User.Side != PlayerSide.Unspecified)
        {
          _registry.ArenaService.UpdateViewForSide(game.User.Side);
        }

        HandleRenderPlayer(PlayerName.User, game.User);
      }

      if (game.Opponent != null)
      {
        HandleRenderPlayer(PlayerName.Opponent, game.Opponent);
      }

      _registry.RaidService.RaidActive = game.RaidActive;
      yield return _registry.CardService.Sync(game.Cards.ToList(), game.GameObjectPositions, animate);

      // Must run after move completion, uses card positions for anchoring
      _registry.DocumentService.RenderMainControls(game.MainControls);
      
      _registry.TutorialService.SetTutorialEffects(game.TutorialEffects);
    }

    void HandleRenderPlayer(PlayerName playerName, PlayerView playerView)
    {
      if (playerView.Side != PlayerSide.Unspecified)
      {
        _registry.GameCharacterForPlayer(playerName).Side = playerView.Side;
      }

      if (playerView.PlayerInfo != null)
      {
        _registry.GameCharacterForPlayer(playerName).RenderPlayerInfo(playerView.PlayerInfo);
      }

      if (playerView.Score != null)
      {
        _registry.GameCharacterForPlayer(playerName).RenderScore(playerView.Score);
      }

      if (playerView.Mana != null)
      {
        _registry.ManaDisplayForPlayer(playerName).RenderManaDisplay(playerView.Mana);
      }

      if (playerView.ActionTracker != null)
      {
        _registry.ActionDisplayForPlayer(playerName).RenderActionTrackerView(playerView.ActionTracker);
      }

      if (playerView.CanTakeAction)
      {
        _registry.CapabilityService.CurrentPriority = playerName;
      }
    }

    IEnumerator HandleLoadScene(LoadSceneCommand command)
    {
      if (command.SkipIfCurrent && command.SceneName.Equals(SceneManager.GetActiveScene().name))
      {
        yield break;
      }
      
      LogUtils.Log($"Loading scene '{command.SceneName}'");
      yield return SceneManager.LoadSceneAsync(command.SceneName, command.Mode switch
      {
        SceneLoadMode.Single => LoadSceneMode.Single,
        SceneLoadMode.Additive => LoadSceneMode.Additive,
        _ => LoadSceneMode.Single
      });
    }

    void HandleClientDebugCommand(ClientDebugCommand command)
    {
      switch (command.DebugCommandCase)
      {
        case ClientDebugCommand.DebugCommandOneofCase.ShowLogs:
          _registry.LogViewer.DoShow();
          break;
        case ClientDebugCommand.DebugCommandOneofCase.InvokeAction:
          StartCoroutine(InvokeActionDelayed(command.InvokeAction));
          break;
        case ClientDebugCommand.DebugCommandOneofCase.LogMessage:
          switch (command.LogMessage.Level)
          {
            case LogMessageLevel.Error:
              LogUtils.LogError(command.LogMessage.Text);
              break;
            case LogMessageLevel.Warning:
              Debug.LogWarning(command.LogMessage.Text);
              break;
            default:
              LogUtils.Log(command.LogMessage.Text);
              break;
          }

          break;
        case ClientDebugCommand.DebugCommandOneofCase.SetBooleanPreference:
          PlayerPrefs.SetInt(command.SetBooleanPreference.Key, command.SetBooleanPreference.Value ? 1 : 0);
          break;
        case ClientDebugCommand.DebugCommandOneofCase.ShowFeedbackForm:
          _registry.UserReportingScript.transform.parent.GetComponent<Canvas>().enabled = true;
          _registry.DocumentService.WaitFor(WaitingFor.FeedbackForm);
          _registry.UserReportingScript.CreateUserReport(() =>
          {
            _registry.DocumentService.EndWaitFor(WaitingFor.FeedbackForm);
          });
          break;
      }
    }

    IEnumerator InvokeActionDelayed(ClientAction action)
    {
      yield return new WaitForSeconds(0.1f);
      _registry.ActionService.HandleAction(action);
    }

    IEnumerator HandleCreateTokenCard(CreateTokenCardCommand command)
    {
      return _registry.CardService.Sync(new List<CardView> { command.Card }, null, command.Animate, delete: false);
    }
  }
}