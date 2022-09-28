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

using System.Collections.Generic;
using System.Linq;
using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine;
using UnityEngine.UIElements;

namespace Spelldawn.Masonry
{
  public sealed class BottomSheet : VisualElement
  {
    readonly VisualElement _sheet;
    readonly Registry _registry;
    readonly List<InterfacePanelAddress> _stack = new();
    VisualElement _content = new NodeVisualElement();
    List<InterfacePanelAddress>? _stackUpdate;
    
    public BottomSheet(Registry registry)
    {
      _registry = registry;
      name = "BottomSheetOverlay";
      style.display = DisplayStyle.None;
      style.backgroundColor = new Color(0f, 0f, 0f, 0.75f);
      style.position = Position.Absolute;
      style.top = 0;
      style.right = 0;
      style.bottom = 0;
      style.left = 0;

      var safeArea = registry.DocumentService.GetSafeArea();
      _sheet = new VisualElement
      {
        name = "BottomSheet",
        style =
        {
          position = Position.Absolute,
          top = safeArea.Top.Value + 44,
          right = safeArea.Right.Value + 16,
          bottom = safeArea.Bottom.Value + 0,
          left = safeArea.Left.Value + 16,
          backgroundColor = Color.gray,
          borderTopLeftRadius = 24,
          borderTopRightRadius = 24,
          justifyContent = Justify.Center,
          alignItems = Align.Center
        }
      };
      
      _sheet.Add(_content);
      Add(_sheet);
    }

    /// <summary>
    /// Opens the bottom sheet to display the panel with 'address' or closes it. Clears all other displayed content. 
    /// </summary>
    public void ToggleOpen(bool open, InterfacePanelAddress address)
    {
      _stackUpdate = open ? new List<InterfacePanelAddress> { address } : new List<InterfacePanelAddress>();
      Debug.Log($"ToggleOpen: toggled with update {_stackUpdate.Count}");
    }
    
    public void TogglePush(bool open, InterfacePanelAddress address)
    {

    }

    public void RefreshPanels(Dictionary<InterfacePanelAddress, Node> panelCache)
    {
      if (_stackUpdate != null)
      {
        style.display = DisplayStyle.Flex;

        if (_stackUpdate.Count > 0 && panelCache.ContainsKey(_stackUpdate.Last()))
        {
          var result = Reconciler.Update(_registry, panelCache[_stackUpdate.Last()], _content);
          if (result != null)
          {
            _sheet.Clear();
            _content = result;
            _sheet.Add(_content);
          }
        }
      }
    }
  }
}