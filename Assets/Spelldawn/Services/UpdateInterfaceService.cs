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
using System.Linq;
using DG.Tweening;
using Spelldawn.Masonry;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.UIElements;
using EasingMode = Spelldawn.Protos.EasingMode;

namespace Spelldawn.Services
{
  public sealed class UpdateInterfaceService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    readonly Dictionary<string, VisualElement> _clones = new();
    readonly Dictionary<string, Rect> _cloneWorldBounds = new();

    public IEnumerator HandleUpdate(UpdateInterfaceCommand command)
    {
      var sequence = TweenUtils.Sequence("UpdateInterface");
      foreach (var step in command.Steps)
      {
        HandleStep(sequence, step);
      }

      _clones.Clear();
      _cloneWorldBounds.Clear();
      yield return sequence.WaitForCompletion();
    }

    void HandleStep(Sequence sequence, UpdateInterfaceStep step)
    {
      var element = FindElement(Errors.CheckNotNull(step.Element));

      switch (Errors.CheckNotNull(step.Update).UpdateCase)
      {
        case InterfaceUpdate.UpdateOneofCase.CloneElement:
          // We check this so that clones happen sequentially before other operations.
          if (step.StartTime.Milliseconds == 0)
          {
            CreateClone(element);
          }
          else
          {
            sequence.InsertCallback(Seconds(step.StartTime), () => { CreateClone(element); });
          }

          break;
        case InterfaceUpdate.UpdateOneofCase.DestroyElement:
          if (step.StartTime.Milliseconds == 0)
          {
            element.RemoveFromHierarchy();
          }
          else
          {
            sequence.InsertCallback(Seconds(step.StartTime), () => { element.RemoveFromHierarchy(); });
          }

          break;
        case InterfaceUpdate.UpdateOneofCase.AnimateToPosition:
          var (t1, t2) = AnimateToPosition(element, step.Element.ElementName, step.Update.AnimateToPosition);
          sequence.Insert(Seconds(step.StartTime), t1);
          sequence.Insert(Seconds(step.StartTime), t2);
          break;
        case InterfaceUpdate.UpdateOneofCase.AnimateToChildIndex:
          break;
        case InterfaceUpdate.UpdateOneofCase.AnimateStyle:
          sequence.Insert(Seconds(step.StartTime), AnimateStyle(element, step.Update.AnimateStyle));
          break;
        default:
          throw new ArgumentOutOfRangeException();
      }
    }

    (Tween, Tween) AnimateToPosition(VisualElement element, string? elementName, AnimateToPosition animateToPosition)
    {
      // When elements are cloned, they may not yet have a valid worldBound. In this situation we need to pull
      // the worldBound from the original element, so we store that information in _cloneWorldBounds.
      var worldBound = elementName != null && _cloneWorldBounds.ContainsKey(elementName)
        ? _cloneWorldBounds[elementName]
        : element.worldBound;
      var target = FindElement(animateToPosition.Destination);
      element.style.position = Position.Absolute;
      element.style.left = worldBound.x;
      element.style.top = worldBound.y;

      // For shrink animations, we need to offset the target position based on the source element
      // size. This is because Unity calculates positions *before* applying scale transformations.      
      var targetPosition = target.worldBound.position -
                           new Vector2(worldBound.width / 2.0f, worldBound.height / 2.0f) -
                           new Vector2(element.style.marginLeft.value.value, element.style.marginTop.value.value) +
                           new Vector2(target.worldBound.width / 2.0f, target.worldBound.height / 2.0f);

      
      return (DOTween.To(() => element.style.left.value.value,
          x => element.style.left = x,
          targetPosition.x,
          Seconds(animateToPosition.Animation.Duration)),
        DOTween.To(() => element.style.top.value.value,
          y => element.style.top = y,
          targetPosition.y,
          Seconds(animateToPosition.Animation.Duration)));
    }

    Tween AnimateStyle(VisualElement element, AnimateElementStyle style)
    {
      Tween tween;
      switch (style.PropertyCase)
      {
        case AnimateElementStyle.PropertyOneofCase.Opacity:
          if (element.style.opacity.keyword == StyleKeyword.Null)
          {
            element.style.opacity = 1.0f;
          }
          tween = DOTween.To(() => element.style.opacity.value,
            x => element.style.opacity = x,
            style.Opacity,
            Seconds(style.Animation.Duration));
          break;
        case AnimateElementStyle.PropertyOneofCase.Width:
          if (element.style.width.keyword == StyleKeyword.Null)
          {
            throw new InvalidOperationException("Element does not have a width");
          }
          var widthUnit = element.style.width.value.unit;
          tween = DOTween.To(() => element.style.width.value.value,
            x => element.style.width = new Length(x, widthUnit),
            style.Width,
            Seconds(style.Animation.Duration));
          break;
        case AnimateElementStyle.PropertyOneofCase.Height:
          if (element.style.height.keyword == StyleKeyword.Null)
          {
            throw new InvalidOperationException("Element does not have a height");
          }          
          var heightUnit = element.style.height.value.unit;
          tween = DOTween.To(() => element.style.height.value.value,
            x => element.style.height = new Length(x, heightUnit),
            style.Height,
            Seconds(style.Animation.Duration));
          break;
        case AnimateElementStyle.PropertyOneofCase.Scale:
          if (element.style.scale.keyword == StyleKeyword.Null)
          {
            element.style.scale = new Scale(Vector3.one);
          }
          tween = DOTween.To(() => element.style.scale.value.value,
            x => element.style.scale = new Scale(x),
            new Vector3(style.Scale.X, style.Scale.Y, 1),
            Seconds(style.Animation.Duration));
          break;
        default:
          throw new ArgumentOutOfRangeException();
      }

      return tween.SetEase(AdaptEase(style.Animation.Ease));
    }

    Ease AdaptEase(EasingMode ease) => ease switch
    {
      EasingMode.EaseIn => Ease.InQuad,
      EasingMode.EaseOut => Ease.OutQuad,
      EasingMode.EaseInOut => Ease.InOutQuad,
      EasingMode.Linear => Ease.Linear,
      EasingMode.EaseInSine => Ease.InSine,
      EasingMode.EaseOutSine => Ease.OutSine,
      EasingMode.EaseInOutSine => Ease.InOutSine,
      EasingMode.EaseInCubic => Ease.InCubic,
      EasingMode.EaseOutCubic => Ease.OutCubic,
      EasingMode.EaseInOutCubic => Ease.InOutCubic,
      EasingMode.EaseInCirc => Ease.InCirc,
      EasingMode.EaseOutCirc => Ease.OutCirc,
      EasingMode.EaseInOutCirc => Ease.InOutCirc,
      EasingMode.EaseInElastic => Ease.InElastic,
      EasingMode.EaseOutElastic => Ease.OutElastic,
      EasingMode.EaseInOutElastic => Ease.InOutElastic,
      EasingMode.EaseInBack => Ease.InBack,
      EasingMode.EaseOutBack => Ease.OutBack,
      EasingMode.EaseInOutBack => Ease.InOutBack,
      EasingMode.EaseInBounce => Ease.InBounce,
      EasingMode.EaseOutBounce => Ease.OutBounce,
      EasingMode.EaseInOutBounce => Ease.InOutBounce,
      _ => Ease.Linear
    };

    void CreateClone(VisualElement input)
    {
      var element = ((IMasonElement)input).Clone(_registry);
      input.style.visibility = Visibility.Hidden;
      var initialPosition = input.worldBound.position;
      element.name = "<Animating>";
      element.style.left = initialPosition.x;
      element.style.top = initialPosition.y;
      element.style.position = Position.Absolute;
      _registry.DocumentService.RootVisualElement.Add(element);

      _clones[input.name] = element;
      _cloneWorldBounds[input.name] = input.worldBound;
    }

    VisualElement FindElement(ElementSelector elementSelector)
    {
      switch (elementSelector.SelectorCase)
      {
        case ElementSelector.SelectorOneofCase.ElementName:
          var elementName = Errors.CheckNotNull(elementSelector.ElementName);
          if (_clones.ContainsKey(elementName))
          {
            return _clones[elementName];
          }
          else
          {
            var results = _registry.DocumentService.RootVisualElement.Query<VisualElement>(elementName).Build();
            Errors.CheckState(results.Count() == 1, $"Expected exactly 1 {elementName} but got {results.Count()}");
            return results.First();
          }
        case ElementSelector.SelectorOneofCase.DragIndicator:
          return _registry.InputService.CurrentDragIndicator();
        default:
          throw new ArgumentOutOfRangeException();
      }
    }

    static float Seconds(Protos.TimeValue value) => value.Milliseconds / 1000f;
  }
}