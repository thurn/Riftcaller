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

using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine.UIElements;

namespace Spelldawn.Masonry
{
  public sealed class Draggable : VisualElement
  {
    readonly Registry _registry;
    readonly Node _node;
    
    public Draggable(Registry registry, Node node)
    {
      _registry = registry;
      _node = node;
      RegisterCallback<MouseDownEvent>(OnMouseDown);
    }

    ~Draggable()
    {
      UnregisterCallback<MouseDownEvent>(OnMouseDown);
    }

    void OnMouseDown(MouseDownEvent evt)
    {
      var dragElement = (Draggable)Mason.Render(_registry, _node);
      _registry.DocumentService.SetCurrentlyDragging(dragElement, worldBound.position);
    }
  }
}