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
using System.Collections.Generic;
using System.Linq;
using Spelldawn.Game;
using Spelldawn.Masonry;
using UnityEngine;
using UnityEngine.UIElements;

namespace Spelldawn.Services
{
  public sealed class InputService : MonoBehaviour
  {
    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];
    Displayable? _lastClicked;
    Draggable? _currentlyDragging;
    VisualElement? _overTargetIndicator;
    Vector2? _dragStartMousePosition; 
    bool _overTarget;
    [SerializeField] Registry _registry = null!;

    public void SetCurrentlyDragging(Draggable element, Vector2 initialPosition)
    {
      element.name = "<DragElement>";
      SetPosition(element, initialPosition);
      element.style.position = Position.Absolute;
      _registry.DocumentService.RootVisualElement.Add(element);
      element.BringToFront();
      element.style.visibility = Visibility.Hidden;
      _currentlyDragging = element;
      _dragStartMousePosition = _registry.DocumentService.ElementMousePosition();

      if (element.OverTargetIndicator != null)
      {
        _overTargetIndicator = Mason.Render(_registry, element.OverTargetIndicator);
        _overTargetIndicator.style.position = Position.Absolute;
        _overTargetIndicator.style.visibility = Visibility.Hidden;
        _registry.DocumentService.RootVisualElement.Add(_overTargetIndicator);
        _overTargetIndicator.BringToFront();
        _overTargetIndicator.name = "<OverTargetIndicator>";
      }
    }

    void Update()
    {
      // I don't trust any of Unity's event handling code. They couldn't event-handle their way
      // out of a wet paper bag.      

      switch (Input.GetMouseButton(0))
      {
        case true when _lastClicked:
          _lastClicked!.MouseDrag();
          break;
        case true when !_lastClicked:
          _lastClicked = FireMouseDown();
          break;
        case false when _lastClicked:
          var last = _lastClicked;
          _lastClicked = null;
          _registry.CardService.ClearInfoZoom();
          last!.MouseUp();
          break;
      }

      switch (Input.GetMouseButton(0))
      {
        case true when _currentlyDragging != null:
          ElementMouseMove(_currentlyDragging);
          break;
        case false when _currentlyDragging != null:
          ElementMouseUp(_currentlyDragging);
          break;
      }
    }

    Displayable? FireMouseDown()
    {
      var ray = _registry.MainCamera.ScreenPointToRay(Input.mousePosition);
      var hits = Physics.RaycastNonAlloc(ray, _raycastHitsTempBuffer, 100);

      var candidates = new List<Displayable>();
      for (var i = 0; i < hits; ++i)
      {
        var hit = _raycastHitsTempBuffer[i];
        var displayable = hit.collider.GetComponent<Displayable>();
        if (displayable && displayable.CanHandleMouseDown())
        {
          candidates.Add(displayable);
        }
      }

      var fired = candidates
        .OrderBy(c => c.GameContext.SortOrder())
        .ThenBy(c => c.SortingKey)
        .ThenBy(c => c.SortingSubkey)
        .LastOrDefault();
      if (fired)
      {
        fired!.MouseDown();
      }

      Array.Clear(_raycastHitsTempBuffer, 0, _raycastHitsTempBuffer.Length);
      return fired;
    }

    void ElementMouseMove(Draggable currentlyDragging)
    {
      var mousePosition = _registry.DocumentService.ElementMousePosition();
      var horizontalDistance = Mathf.Abs(mousePosition.x - _dragStartMousePosition!.Value.x);
      if (_currentlyDragging!.HorizontalDragStartDistance is {} distance && horizontalDistance < distance)
      {
        _currentlyDragging!.style.visibility = Visibility.Hidden;
        if (_overTargetIndicator != null)
        {
          _overTargetIndicator.style.visibility = Visibility.Hidden;
        }
        return;
      }

      var dropTargets = _registry.DocumentService.RootVisualElement.Query<DropTarget>().Build().ToList();
      var dragger = (_overTarget && _overTargetIndicator != null) ? _overTargetIndicator : currentlyDragging;

      var target = dropTargets.Where(target =>
          target.worldBound.Contains(mousePosition) &&
          currentlyDragging.TargetIdentifiers.Contains(target.name))
        .OrderBy(x =>
          Vector2.Distance(x.worldBound.position,
            dragger.worldBound.position)).FirstOrDefault();
      _overTarget = target != null;

      if (target != null && _overTargetIndicator != null)
      {
        currentlyDragging.style.visibility = Visibility.Hidden;
        _overTargetIndicator.style.visibility = Visibility.Visible;
        SetPosition(_overTargetIndicator, GetMousePosition(_overTargetIndicator));
      }
      else
      {
        currentlyDragging.style.visibility = Visibility.Visible;
        if (_overTargetIndicator != null)
        {
          _overTargetIndicator.style.visibility = Visibility.Hidden;
        }

        SetPosition(currentlyDragging, GetMousePosition(currentlyDragging));
      }
    }

    void ElementMouseUp(Draggable currentlyDragging)
    {
      // Leave the currently-visible drag object in the hierarchy, the OnDrop action is responsible for removing it.
      
      if (currentlyDragging.style.visibility == Visibility.Hidden)
      {
        currentlyDragging.RemoveFromHierarchy();
      }

      if (_overTargetIndicator?.style.visibility == Visibility.Hidden)
      {
        _overTargetIndicator.RemoveFromHierarchy();
      }

      if (currentlyDragging.OnDrop != null && _overTarget)
      {
        _registry.ActionService.HandleAction(currentlyDragging.OnDrop);
      }

      _currentlyDragging = null;
      _overTargetIndicator = null;
    }

    Vector2 GetMousePosition(VisualElement element)
    {
      var position = _registry.DocumentService.ScreenPositionToElementPosition(Input.mousePosition);
      return new Vector2(
        position.Left - (element.layout.width / 2),
        position.Top - (element.layout.height / 2));
    }

    void SetPosition(VisualElement element, Vector2 pos)
    {
      element.style.left = pos.x;
      element.style.top = pos.y;
    }
  }
}