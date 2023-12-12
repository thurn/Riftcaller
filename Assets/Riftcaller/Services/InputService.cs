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

using System;
using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Riftcaller.Game;
using Riftcaller.Masonry;
using Riftcaller.Protos;
using Riftcaller.Utils;
using UnityEngine;
using UnityEngine.UIElements;

namespace Riftcaller.Services
{
  public sealed class InputService : MonoBehaviour
  {
    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];
    readonly List<KeyboardMapping> _keyboardMappings = new();
    [SerializeField] Displayable? _lastClicked;
    [SerializeField] Displayable? _lastHovered;
    Draggable? _originalDragSource;
    VisualElement? _currentlyDragging;
    VisualElement? _overTargetIndicator;
    Vector2? _dragStartMousePosition;
    bool _overTarget;
    [SerializeField] Registry _registry = null!;

    const string DragElementName = "<DragElement>";
    const string OverTargetIndicatorElementName = "<OverTargetIndicator>";

    public void StartDragging(Draggable newDragSource)
    {
      _originalDragSource = newDragSource;
      VisualElement element;
      if (newDragSource.CustomDragIndicator is { } indicator)
      {
        var draggable = Mason.Render(_registry, indicator);
        element = draggable;
      }
      else
      {
        var draggable = (Draggable)((IMasonElement)newDragSource).Clone(_registry);
        element = draggable;
      }

      var initialPosition = newDragSource.worldBound.position;
      element.name = DragElementName;
      SetPosition(element, initialPosition);
      element.style.position = Position.Absolute;
      _registry.DocumentService.RootVisualElement.Add(element);
      element.BringToFront();
      element.style.visibility = Visibility.Hidden;

      _currentlyDragging = element;
      _dragStartMousePosition = _registry.DocumentService.ElementMousePosition();
      _overTarget = false;

      if (newDragSource.OverTargetIndicator != null)
      {
        _overTargetIndicator = Mason.Render(_registry, newDragSource.OverTargetIndicator);
        _overTargetIndicator.style.position = Position.Absolute;
        _overTargetIndicator.style.visibility = Visibility.Hidden;
        _registry.DocumentService.RootVisualElement.Add(_overTargetIndicator);
        _overTargetIndicator.BringToFront();
        _overTargetIndicator.name = OverTargetIndicatorElementName;
      }
    }

    /// <summary>Returns the interface element currently being dragged.</summary>
    public VisualElement? CurrentDragIndicator()
    {
      var overTargetElement = _registry.DocumentService.RootVisualElement.Q(OverTargetIndicatorElementName);
      return overTargetElement ?? _registry.DocumentService.RootVisualElement.Q(DragElementName);
    }

    public void SetKeyboardShortcuts(IEnumerable<KeyboardMapping> mapping)
    {
      _keyboardMappings.Clear();
      _keyboardMappings.AddRange(mapping);
    }

    void Update()
    {
      // I don't trust any of Unity's event handling code. They couldn't event-handle their way
      // out of a wet paper bag.
      
      HandleDisplayableClickAndDrag();
      HandleDisplayableHover();
      HandleVisualElementDrag();
      HandleKeyboardShortcuts();
    }

    void HandleVisualElementDrag()
    {
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

    void HandleDisplayableClickAndDrag()
    {
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
          last!.MouseUp();
          break;
      }
    }

    void HandleDisplayableHover()
    {
      if (Input.GetMouseButton(0))
      {
        return;
      }
      
      var current = ObjectAtMousePosition();
      if (current && current != null && !_lastHovered)
      {
        current.MouseHoverStart();
        _lastHovered = current;
      }
      else if (!current && _lastHovered && _lastHovered != null)
      {
        _lastHovered.MouseHoverEnd();
        _lastHovered = null;
      }
      else if (current && current != null && _lastHovered && _lastHovered != null && current != _lastHovered)
      {
        _lastHovered.MouseHoverEnd();
        current.MouseHoverStart();
        _lastHovered = current;
      }
      else if (current && current != null && current == _lastHovered)
      {
        current.MouseHover();
      }
    }

    void HandleKeyboardShortcuts()
    {
      foreach (var mapping in _keyboardMappings)
      {
        if (Input.GetKeyDown(mapping.Shortcut.KeyName))
        {
          if (mapping.Shortcut.Alt && !(Input.GetKey(KeyCode.LeftAlt) || Input.GetKey(KeyCode.RightAlt)))
          {
            continue;
          }
          if (mapping.Shortcut.Ctrl && !(Input.GetKey(KeyCode.LeftControl) || Input.GetKey(KeyCode.RightControl)))
          {
            continue;
          }
          if (mapping.Shortcut.Shift && !(Input.GetKey(KeyCode.LeftShift) || Input.GetKey(KeyCode.RightShift)))
          {
            continue;
          }
          
          _registry.ActionService.HandleAction(mapping.Action);
        }
      }
      
    }

    Displayable? FireMouseDown()
    {
      Debug.Log($"FireMouseDown");
      if (_registry.DocumentService.IsAnyPanelOpen() ||
          _registry.DocumentService.MouseOverScreenElement())
      {
        Debug.Log($"FireMouseDown: MouseOverScreenElement");
        return null;
      }
      
      var fired = ObjectAtMousePosition();

      if (fired && fired != null)
      {
        fired.MouseDown();
      }

      return fired;
    }

    Displayable? ObjectAtMousePosition()
    {
      var ray = _registry.MainCamera.ScreenPointToRay(Input.mousePosition);
      var hits = Physics.RaycastNonAlloc(ray, _raycastHitsTempBuffer, 100);

      var candidates = new List<Displayable>();
      for (var i = 0; i < hits; ++i)
      {
        var hit = _raycastHitsTempBuffer[i];
        var displayable = hit.collider.GetComponent<Displayable>();
        if (displayable && displayable.CanHandleMouseEvents())
        {
          candidates.Add(displayable);
        }
      }

      Array.Clear(_raycastHitsTempBuffer, 0, _raycastHitsTempBuffer.Length);
      var fired = candidates
        .OrderBy(c => c.GameContext.SortOrder())
        .ThenBy(c => c.SortingKey)
        .ThenBy(c => c.SortingSubkey)
        .LastOrDefault();
      return fired;
    }

    void ElementMouseMove(VisualElement currentlyDragging)
    {
      var mousePosition = _registry.DocumentService.ElementMousePosition();
      var horizontalDistance = Mathf.Abs(mousePosition.x - _dragStartMousePosition!.Value.x);
      if (_originalDragSource is { HorizontalDragStartDistance: { } distance } && horizontalDistance < distance)
      {
        
        _currentlyDragging!.style.visibility = Visibility.Hidden;
        if (_overTargetIndicator != null)
        {
          _overTargetIndicator.style.visibility = Visibility.Hidden;
        }

        _overTarget = false;
        if (_originalDragSource?.RemoveOriginal == true)
        {
          _originalDragSource.style.visibility = Visibility.Visible;
        }

        return;
      }

      _originalDragSource?.OnDragged();
      
      if (_originalDragSource?.RemoveOriginal == true)
      {
        _originalDragSource.style.visibility = Visibility.Hidden;
      }

      var dropTargets = _registry.DocumentService.RootVisualElement.Query<DropTarget>().Build().ToList();
      var dragger = (_overTarget && _overTargetIndicator != null) ? _overTargetIndicator : currentlyDragging;

      var target = dropTargets.Where(target =>
          target.worldBound.Contains(mousePosition) &&
          _originalDragSource != null &&
          _originalDragSource.TargetIdentifiers.Contains(target.name))
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

    void ElementMouseUp(VisualElement currentlyDragging)
    {
      if (currentlyDragging.style.visibility == Visibility.Hidden)
      {
        currentlyDragging.RemoveFromHierarchy();
      }

      if (_overTargetIndicator?.style.visibility == Visibility.Hidden)
      {
        _overTargetIndicator.RemoveFromHierarchy();
      }

      if (_originalDragSource is { OnDrop: { } } drag && _overTarget)
      {
        // Leave the currently-visible drag object in the hierarchy, the OnDrop action is responsible for removing it.
        _registry.ActionService.HandleAction(drag.OnDrop);
        if (_originalDragSource?.RemoveOriginal == true)
        {
          TweenUtils.Sequence("RemoveDragOriginal").Append(
              DOTween.To(() => _originalDragSource.style.height.value.value,
                height => _originalDragSource.style.height = height,
                endValue: 0,
                0.3f))
            .AppendCallback(_originalDragSource.RemoveFromHierarchy);
        }
      }
      else
      {
        _overTargetIndicator?.RemoveFromHierarchy();
        _registry.UpdateInterfaceService.MoveElementToPosition(
          currentlyDragging,
          Errors.CheckNotNull(_originalDragSource),
          new Protos.TimeValue { Milliseconds = 100 }, 
          () =>
          {
            if (_originalDragSource?.RemoveOriginal == true)
            {
              _originalDragSource.style.visibility = Visibility.Visible;
            }
            currentlyDragging.RemoveFromHierarchy();
          });
      }

      _currentlyDragging = null;
      _overTargetIndicator = null;
      _overTarget = false;
    }

    Vector2? GetMousePosition(VisualElement element)
    {
      if (!(float.IsNormal(element.layout.width) && float.IsNormal(element.layout.height)))
      {
        return null;
      }
      
      var position = _registry.DocumentService.ScreenPositionToElementPosition(
        DocumentService.DefaultScreenMode, Input.mousePosition);
      return new Vector2(
        position.Left - (element.layout.width / 2),
        position.Top - (element.layout.height / 2));
    }

    void SetPosition(VisualElement element, Vector2? position)
    {
      if (position is {} pos)
      {
        Errors.CheckFloat(pos.x);
        Errors.CheckFloat(pos.y);
        element.style.left = pos.x;
        element.style.top = pos.y;        
      }
    }
  }
}