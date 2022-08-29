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
using Google.Protobuf.WellKnownTypes;
using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class TogglePanelButton : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] Panel _panel;
    float? _pressTime;
    bool _wasLongPress;

    enum Panel
    {
      GameMenu,
      Feedback
    }

    void OnMouseDown()
    {
      _pressTime = Time.time;
      _wasLongPress = false;
    }

    void OnMouseDrag()
    {
      if (_pressTime is { } time && !_wasLongPress && (Time.time - time) > 1)
      {
        _wasLongPress = true;
        StartCoroutine(_registry.CommandService.HandleCommands(new GameCommand
        {
          Debug = new ClientDebugCommand
          {
            ShowLogs = new Empty()
          }
        }));        
      }
    }

    void OnMouseUpAsButton()
    {
      _registry.StaticAssets.PlayButtonSound();
      _pressTime = null;

      if (!_wasLongPress)
      {
        var address = new InterfacePanelAddress
        {
          ClientPanel = _panel switch
          {
            Panel.GameMenu => throw new NotImplementedException(),
            Panel.Feedback => ClientPanelAddress.DebugPanel,
            _ => throw new ArgumentOutOfRangeException()
          }
        };
      
        _registry.DocumentService.TogglePanel(!_registry.DocumentService.IsOpen(address), address);        
      }
    }
  }
}