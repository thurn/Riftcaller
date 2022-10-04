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

using System;
using System.Collections;
using System.Collections.Generic;
using System.IO;
using System.Net.Http;
using System.Text;
using Grpc.Core;
using Grpc.Net.Client;
using Grpc.Net.Client.Web;
using Grpc.Net.Compression;
using Spelldawn.Protos;
using Spelldawn.Tools;
using Spelldawn.Utils;
using UnityEngine;
using CompressionLevel = System.IO.Compression.CompressionLevel;
using Random = UnityEngine.Random;

namespace Spelldawn.Services
{
  public sealed class ActionService : MonoBehaviour
  {
    const string ServerAddress = "http://localhost:50052";

    readonly Protos.Spelldawn.SpelldawnClient _client = new(GrpcChannel.ForAddress(
      ServerAddress, new GrpcChannelOptions
      {
        HttpHandler = new GrpcWebHandler(new HttpClientHandler()),
        Credentials = ChannelCredentials.Insecure,
        CompressionProviders = new List<ICompressionProvider>
        {
          new GzipCompressionProvider(CompressionLevel.Optimal)
        }
      }));

    [SerializeField] Registry _registry = null!;
    [SerializeField] bool _currentlyHandlingAction;
    readonly Queue<ClientAction> _actionQueue = new();
    PlayerIdentifier? _playerIdentifier;
    bool _attemptReconnect;

    public bool OfflineMode { get; private set; }

    public bool Active => _currentlyHandlingAction || _actionQueue.Count > 0;

    void Start()
    {
      _attemptReconnect = false;
      StartCoroutine(AutoReconnect());
    }

    public void Connect(PlayerIdentifier playerIdentifier, bool offlineMode)
    {
      _playerIdentifier = playerIdentifier;
      OfflineMode = offlineMode;
      AttemptConnection();
    }

    public void HandleAction(ClientAction action)
    {
      if (!_registry.CapabilityService.CanExecuteAction(action.ActionCase))
      {
        var message = new StringBuilder();
        message.Append($"Error: User cannot currently perform action {action}");
        throw new InvalidOperationException(message.ToString());
      }

      ApplyImmediateResponse(action);
      _actionQueue.Enqueue(action);
    }

    void Update()
    {
      if (_actionQueue.Count > 0 && !_currentlyHandlingAction)
      {
        _currentlyHandlingAction = true;
        StartCoroutine(HandleActionAsync(_actionQueue.Dequeue()));
      }

#if USE_UNITY_PLUGIN
      var pollCommands = Plugin.Poll();
      if (pollCommands != null)
      { 
        StartCoroutine(_registry.CommandService.HandleCommands(pollCommands));
      }
#endif
    }

    IEnumerator AutoReconnect()
    {
      while (true)
      {
        yield return new WaitForSeconds(1.0f);
        if (_attemptReconnect)
        {
          AttemptConnection();
        }
      }

      // ReSharper disable once IteratorNeverReturns
    }

    async void AttemptConnection()
    {
      var request = new ConnectRequest
      {
        PlayerId = Errors.CheckNotNull(_playerIdentifier),
      };

      if (OfflineMode)
      {
        Debug.Log($"Connecting to Offline Game");
        StartCoroutine(ConnectToOfflineGame(request));
      }
      else
      {
        // TODO: Android in particular seems to hang for multiple minutes when the server can't be reached?
        using var call = _client.Connect(request);

        try
        {
          while (await call.ResponseStream.MoveNext())
          {
            if (this != null)
            {
              var commands = call.ResponseStream.Current;
              _registry.DocumentService.Loading = false;
              _attemptReconnect = false;
              StartCoroutine(_registry.CommandService.HandleCommands(commands));
              _registry.DocumentService.FetchOpenPanels();
            }
          }
        }
        catch (RpcException e)
        {
          _registry.DocumentService.Loading = true;
          _attemptReconnect = true;
          if (!DoNotLogRpcErrors.ShouldSkipLoggingRpcErrors)
          {
            Debug.Log($"RpcException: {e.StatusCode} -- {e.Message}");
          }
        }
      }
    }

    /// <summary>Connects to an existing offline game, handling responses.</summary>
    public IEnumerator ConnectToOfflineGame(ConnectRequest request)
    {
#if USE_UNITY_PLUGIN
      var commands = Plugin.Connect(request);
      if (commands != null)
      {
        yield return _registry.CommandService.HandleCommands(commands);
      }
#else
      Debug.LogError("Plugin not enabled");
      yield break;
#endif
    }

    IEnumerator HandleActionAsync(ClientAction action)
    {
      StartCoroutine(ApplyOptimisticResponse(action));
      if (action.ActionCase == ClientAction.ActionOneofCase.StandardAction)
      {
        if (action.StandardAction.Payload.Length == 0)
        {
          // No need to send empty payload to server
          _currentlyHandlingAction = false;
          yield break;          
        }

        _registry.DocumentService.AddRequestFields(action.StandardAction);
      }

      // Introduce simulated server delay
      if (IntroduceNetworkDelay.ShouldIntroduceNetworkDelay)
      {
        yield return new WaitForSeconds(5f);
      }
      else if (!NoNetworkDelay.ShouldRemoveNetworkDelay)
      {
        yield return new WaitForSeconds(Random.Range(0f, 0.5f));
      }

      // Send to server
      var request = new GameRequest
      {
        Action = action,
        PlayerId = Errors.CheckNotNull(_playerIdentifier),
      };
      request.OpenPanels.AddRange(_registry.DocumentService.OpenPanels);

      if (OfflineMode)
      {
#if USE_UNITY_PLUGIN
        yield return _registry.CommandService.HandleCommands(Plugin.PerformAction(request));
#else
        Debug.LogError("Plugin not enabled");
#endif
      }
      else
      {
        var call = _client.PerformActionAsync(request);
        var task = call.GetAwaiter();
        yield return new WaitUntil(() => task.IsCompleted);

        switch (call.GetStatus().StatusCode)
        {
          case StatusCode.OK:
            _registry.DocumentService.Loading = false;
            _attemptReconnect = false;
            yield return _registry.CommandService.HandleCommands(task.GetResult());
            break;
          default:
            _registry.DocumentService.Loading = true;
            _attemptReconnect = true;
            if (!DoNotLogRpcErrors.ShouldSkipLoggingRpcErrors)
            {
              Debug.Log($"Error connecting to {ServerAddress}: {call.GetStatus().Detail}");
            }

            break;
        }
      }

      _currentlyHandlingAction = false;
    }

    /// <summary>
    /// Immediate action handling, without waiting for the queue. This is needed to avoid things that feel
    /// broken, like waiting for animations before closing a panel.
    /// </summary>
    void ApplyImmediateResponse(ClientAction action)
    {
      switch (action.ActionCase)
      {
        case ClientAction.ActionOneofCase.StandardAction:
          _registry.StaticAssets.PlayButtonSound();
          if (action.StandardAction.Update is { } update)
          {
            foreach (var command in update.Commands)
            {
              switch (command.CommandCase)
              {
                case GameCommand.CommandOneofCase.TogglePanel:
                    _registry.DocumentService.TogglePanel(command.TogglePanel);
                  break;
              }
            }
          }

          break;
      }
    }

    IEnumerator ApplyOptimisticResponse(ClientAction action)
    {
      switch (action.ActionCase)
      {
        case ClientAction.ActionOneofCase.StandardAction:
          if (action.StandardAction.Update is { } update)
          {
            yield return _registry.CommandService.HandleCommands(update);
          }

          break;
        case ClientAction.ActionOneofCase.DrawCard:
          _registry.StaticAssets.PlayDrawCardStartSound();
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          _registry.CardService.DrawOptimisticCard();
          break;
        case ClientAction.ActionOneofCase.PlayCard:
          yield return HandlePlayCard(action.PlayCard);
          break;
        case ClientAction.ActionOneofCase.GainMana:
          _registry.StaticAssets.PlayAddManaSound();
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          _registry.ManaDisplayForPlayer(PlayerName.User).GainMana(1);
          break;
        case ClientAction.ActionOneofCase.InitiateRaid:
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          yield return _registry.CommandService.HandleCommands(new GameCommand
          {
            VisitRoom = new VisitRoomCommand
            {
              RoomId = action.InitiateRaid.RoomId,
              Initiator = PlayerName.User,
              VisitType = RoomVisitType.InitiateRaid
            }
          });
          break;
        case ClientAction.ActionOneofCase.LevelUpRoom:
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          yield return _registry.CommandService.HandleCommands(new GameCommand
          {
            VisitRoom = new VisitRoomCommand
            {
              RoomId = action.LevelUpRoom.RoomId,
              Initiator = PlayerName.User,
              VisitType = RoomVisitType.LevelUpRoom
            }
          });
          break;
        default:
          yield break;
      }
    }

    IEnumerator HandlePlayCard(PlayCardAction action)
    {
      var card = _registry.CardService.FindCard(action.CardId);
      _registry.StaticAssets.PlayWhooshSound();
      if (card.ReleasePosition == null)
      {
        yield break;
      }

      var position = card.ReleasePosition;

      if (position.PositionCase == ObjectPosition.PositionOneofCase.Room)
      {
        var room = action.Target.RoomId;
        Errors.CheckArgument(room != RoomIdentifier.Unspecified, "No RoomId target provided!");
        // Move to targeted room
        var newPosition = new ObjectPosition();
        newPosition.MergeFrom(position);
        newPosition.Room.RoomId = room;
        position = newPosition;
      }

      yield return _registry.ObjectPositionService.MoveGameObject(card, position);
    }
  }

  /// <summary>
  /// You can use this type instead of 'GzipCompressionProvider' above to log the size of server payloads before
  /// decompression.
  /// </summary>
  // ReSharper disable once UnusedType.Global
  sealed class DebugGzipCompressionProvider : ICompressionProvider
  {
    readonly GzipCompressionProvider _wrappedProvider;

    public DebugGzipCompressionProvider(CompressionLevel defaultCompressionLevel)
    {
      _wrappedProvider = new GzipCompressionProvider(defaultCompressionLevel);
    }

    public Stream CreateCompressionStream(Stream stream, CompressionLevel? compressionLevel) =>
      _wrappedProvider.CreateCompressionStream(stream, compressionLevel);

    public Stream CreateDecompressionStream(Stream stream)
    {
      Debug.Log($">>> Decompressing: {stream.Length}");
      return _wrappedProvider.CreateDecompressionStream(stream);
    }

    public string EncodingName => _wrappedProvider.EncodingName;
  }
}