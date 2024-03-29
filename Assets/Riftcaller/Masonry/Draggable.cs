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

#nullable enable

using System.Collections.Generic;
using System.Linq;
using Riftcaller.Protos;
using Riftcaller.Services;
using UnityEngine.UIElements;

namespace Riftcaller.Masonry
{
  public sealed class Draggable : VisualElement, IMasonElement
  {
    public Registry Registry { get; set; }
    public Node? Node { get; set; }
    public List<string> TargetIdentifiers { get; set; }
    public Node? OverTargetIndicator { get; set; }
    public ClientAction? OnDrop { get; set; }
    public uint? HorizontalDragStartDistance { get; set; }
    public bool RemoveOriginal { get; set; }
    public List<string> HideIndicatorChildren { get; set; }
    public Node? CustomDragIndicator { get; set; }
    public ClientAction? OnDragDetected { get; set; }
    bool _firedDragDetected;

    public static void Apply(Registry registry, Draggable view, Node data)
    {
      view.Registry = registry;
      view.Node = data;
      view.TargetIdentifiers = data.NodeType.DraggableNode.DropTargetIdentifiers.ToList();
      view.OverTargetIndicator = data.NodeType.DraggableNode.OverTargetIndicator;
      view.OnDrop = data.NodeType.DraggableNode.OnDrop;
      view.HorizontalDragStartDistance = data.NodeType.DraggableNode.HorizontalDragStartDistance;
      view.RemoveOriginal = data.NodeType.DraggableNode.RemoveOriginal;
      view.HideIndicatorChildren = data.NodeType.DraggableNode.HideIndicatorChildren.ToList();
      view.CustomDragIndicator = data.NodeType.DraggableNode.CustomDragIndicator;
      view.OnDragDetected = data.NodeType.DraggableNode.OnDragDetected;
    }

    public Draggable()
    {
      Registry = null!;
      Node = null!;
      TargetIdentifiers = new List<string>();
      RegisterCallback<MouseDownEvent>(OnMouseDown);
      HideIndicatorChildren = new List<string>();
    }

    ~Draggable()
    {
      UnregisterCallback<MouseDownEvent>(OnMouseDown);
    }

    void OnMouseDown(MouseDownEvent evt)
    {
      if (OnDrop != null && Registry.CapabilityService.CanDragInterfaceElement())
      {
        _firedDragDetected = false;
        Registry.InputService.StartDragging(this);        
      }
    }

    public void OnDragged()
    {
      if (!_firedDragDetected)
      {
        foreach (var child in HideIndicatorChildren)
        {
          if (this.Q<VisualElement>(child) is { } element)
          {
            element.style.visibility = Visibility.Hidden;
          }
        }

        if (OnDragDetected != null)
        {
          Registry.ActionService.HandleAction(OnDragDetected);
        }
        _firedDragDetected = true;
      }
    }
  }
}