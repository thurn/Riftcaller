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
using System.Collections.Generic;
using Riftcaller.Protos;
using Riftcaller.Services;
using Riftcaller.Utils;
using UnityEngine;
using UnityEngine.UIElements;

#nullable enable

namespace Riftcaller.Masonry
{
  public interface IMasonElement
  {
    Node? Node { get; set; }

    NodeType.NodeTypeOneofCase NodeType() => Node?.NodeType?.NodeTypeCase ?? Protos.NodeType.NodeTypeOneofCase.None;

    VisualElement Clone(Registry registry) => Mason.Render(registry, Errors.CheckNotNull(Node, "Node is null"));
  }

  public interface INodeCallbacks : IMasonElement
  {
    VisualElement Self { get; }
    Lazy<Callbacks> Callbacks { get; }

    void SetCallback(Callbacks.Event eventType, Action? callback)
    {
      if (callback != null || Callbacks.IsValueCreated)
      {
        Callbacks.Value.SetCallback(Self, eventType, callback);
      }
    }
  }

  public sealed class Callbacks
  {
    public enum Event
    {
      Click,
      MouseDown,
      MouseUp,
      MouseEnter,
      MouseLeave,
      LongPress,
      Change,
      FieldChanged
    }

    readonly HashSet<Event> _registered = new();
    readonly Dictionary<Event, Action?> _actions = new();
    bool _mouseDown;
    bool _firedLongPress;

    public void SetCallback(VisualElement e, Event eventType, Action? callback)
    {
      if (!_registered.Contains(eventType))
      {
        _registered.Add(eventType);
        Register(e, eventType);
      }

      _actions[eventType] = callback;
    }

    public bool HasCallback(Event eventType) => _registered.Contains(eventType);

    void Register(VisualElement e, Event eventType)
    {
      switch (eventType)
      {
        case Event.Click:
          e.RegisterCallback<ClickEvent>(OnClick);
          break;
        case Event.MouseEnter:
          e.RegisterCallback<MouseEnterEvent>(OnMouseEnter);
          break;
        case Event.MouseLeave:
          e.RegisterCallback<MouseLeaveEvent>(OnMouseLeave);
          break;
        case Event.MouseDown:
        case Event.LongPress:
          e.RegisterCallback<MouseDownEvent>(OnMouseDown);
          break;
        case Event.MouseUp:
          e.RegisterCallback<MouseUpEvent>(OnMouseUp);
          break;              
        case Event.Change:
          e.RegisterCallback<ChangeEvent<float>>(OnChange);
          break;
        case Event.FieldChanged:
          e.RegisterCallback<ChangeEvent<string>>(OnFieldChange);
          break;
        default:
          throw new ArgumentOutOfRangeException(nameof(eventType), eventType, "Unknown event type");
      }
    }

    public void OnClick(ClickEvent evt)
    {
      if (!_firedLongPress)
      {
        _actions.GetValueOrDefault(Event.Click)?.Invoke();
      }
      
      _firedLongPress = false;
    }

    void OnMouseDown(MouseDownEvent evt)
    {
      _mouseDown = true;
      _actions.GetValueOrDefault(Event.MouseDown)?.Invoke();
      TweenUtils.ExecuteAfter(0.5f, () =>
      {
        if (_mouseDown)
        {
          _firedLongPress = true;
          _actions.GetValueOrDefault(Event.LongPress)?.Invoke();
        }
      });
    }

    void OnMouseUp(MouseUpEvent evt)
    {
      _mouseDown = false;
      _actions.GetValueOrDefault(Event.MouseUp)?.Invoke();
    }

    void OnMouseEnter(MouseEnterEvent evt)
    {
      _actions.GetValueOrDefault(Event.MouseEnter)?.Invoke();
    }

    void OnMouseLeave(MouseLeaveEvent evt)
    {
      _actions.GetValueOrDefault(Event.MouseLeave)?.Invoke();
    }

    void OnChange(ChangeEvent<float> evt)
    {
      _actions.GetValueOrDefault(Event.Change)?.Invoke();
    }
    
    void OnFieldChange(ChangeEvent<string> evt)
    {
      _actions.GetValueOrDefault(Event.FieldChanged)?.Invoke();
    }
  }

  public sealed class NodeVisualElement : VisualElement, INodeCallbacks
  {
    public VisualElement Self => this;
    readonly Lazy<Callbacks> _callbacks = new();
    public Lazy<Callbacks> Callbacks => _callbacks;
    public Node? Node { get; set; }
  }

  public sealed class NodeLabel : Label, INodeCallbacks
  {
    public VisualElement Self => this;
    readonly Lazy<Callbacks> _callbacks = new();
    public Lazy<Callbacks> Callbacks => _callbacks;
    public Node? Node { get; set; }
  }
}