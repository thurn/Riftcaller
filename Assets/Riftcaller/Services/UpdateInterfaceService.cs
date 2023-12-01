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
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Riftcaller.Masonry;
using Riftcaller.Protos;
using Riftcaller.Utils;
using UnityEngine;
using UnityEngine.UIElements;
using EasingMode = Riftcaller.Protos.EasingMode;
using TimeValue = Riftcaller.Protos.TimeValue;

namespace Riftcaller.Services
{
  public sealed class UpdateInterfaceService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    readonly Dictionary<string, VisualElement> _clones = new();
    readonly Dictionary<string, VisualElement> _targets = new();

    public IEnumerator HandleUpdate(UpdateInterfaceCommand command)
    {
      var sequence = TweenUtils.Sequence("UpdateInterface").Pause();
      foreach (var step in command.Steps)
      {
        yield return ApplyUpdate(
          sequence, step, Errors.CheckNotNull(step.Update), FindElement(Errors.CheckNotNull(step.Element)));
      }

      _clones.Clear();

      sequence.TogglePause();
      yield return sequence.WaitForCompletion();
    }

    IEnumerator ApplyUpdate(Sequence sequence, UpdateInterfaceStep step, InterfaceUpdate update, VisualElement element)
    {
      switch (update.UpdateCase)
      {
        case InterfaceUpdate.UpdateOneofCase.CloneElement:
          AddCallbackOrInvoke(sequence, step.StartTime, () => { CreateClone(element); });
          // Wait for clone to be added to the hierarchy.
          yield return new WaitForEndOfFrame();          
          break;
        case InterfaceUpdate.UpdateOneofCase.DestroyElement:
          AddCallbackOrInvoke(sequence, step.StartTime, element.RemoveFromHierarchy);
          break;
        case InterfaceUpdate.UpdateOneofCase.AnimateToPosition:
          var (t1, t2) = AnimateToPosition(element, update.AnimateToPosition);
          sequence.Insert(Seconds(step.StartTime), t1);
          sequence.Insert(Seconds(step.StartTime), t2);
          break;
        case InterfaceUpdate.UpdateOneofCase.ApplyStyle:
          AddCallbackOrInvoke(sequence, step.StartTime,
            () => { Mason.ApplyStyle(_registry, element, update.ApplyStyle); });
          break;
        case InterfaceUpdate.UpdateOneofCase.AnimateStyle:
          sequence.Insert(Seconds(step.StartTime), AnimateStyle(element, update.AnimateStyle));
          break;
        case InterfaceUpdate.UpdateOneofCase.CreateTargetAtChildIndex:
          AddCallbackOrInvoke(sequence, step.StartTime,
            () => { CreateTargetAtChildIndex(element, update.CreateTargetAtChildIndex); });
          // Wait for the target to be added to the hierarchy.
          yield return new WaitForEndOfFrame();
          break;
        default:
          throw new ArgumentOutOfRangeException();
      }
    }

    (Tween, Tween) AnimateToPosition(VisualElement element, AnimateToPosition animateToPosition)
    {
      var target = FindElement(animateToPosition.Destination);
      return RunAnimateToPosition(element, target, animateToPosition);
    }

    /// <summary>Moves the 'source' element to the position of the 'target' element.</summary>
    public void MoveElementToPosition(VisualElement source, VisualElement target, TimeValue duration, Action onComplete)
    {
      var (t1, t2) = RunAnimateToPosition(source, target, new AnimateToPosition
      {
        Animation = new ElementAnimation
        {
          Ease = EasingMode.Linear,
          Duration = duration
        }
      });

      TweenUtils.Sequence("MoveElementToPosition").Insert(0, t1).Insert(0, t2).AppendCallback(() => onComplete());
    }

    (Tween, Tween) RunAnimateToPosition(
      VisualElement element,
      VisualElement target,
      AnimateToPosition animateToPosition)
    {
      var worldBound = element.worldBound;
      Errors.CheckState(float.IsNormal(worldBound.width) && float.IsNormal(worldBound.height),
        "Element does not have a defined size");

      Errors.CheckState(float.IsNormal(target.worldBound.width) && float.IsNormal(target.worldBound.height),
        "Target does not have a defined size");

      element.style.position = Position.Absolute;
      element.style.left = worldBound.x;
      element.style.top = worldBound.y;

      // Animate to align the center of the source element with the center of the target element        
      var targetPosition = target.worldBound.position -
                           new Vector2(
                             animateToPosition.DisableWidthHalfOffset ? 0 : worldBound.width / 2.0f,
                             animateToPosition.DisableHeightHalfOffset ? 0 : worldBound.height / 2.0f) -
                           new Vector2(element.style.marginLeft.value.value, element.style.marginTop.value.value) +
                           new Vector2(target.worldBound.width / 2.0f, target.worldBound.height / 2.0f);

      return (DOTween.To(() => element.style.left.value.value,
            x => element.style.left = x,
            targetPosition.x,
            Seconds(animateToPosition.Animation.Duration)).SetEase(AdaptEase(animateToPosition.Animation.Ease)),
          DOTween.To(() => element.style.top.value.value,
            y => element.style.top = y,
            targetPosition.y,
            Seconds(animateToPosition.Animation.Duration)).SetEase(AdaptEase(animateToPosition.Animation.Ease))
        );
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

    void CreateTargetAtChildIndex(
      VisualElement element,
      CreateTargetAtChildIndex createTarget)
    {
      Errors.CheckState(float.IsNormal(element.worldBound.width) && float.IsNormal(element.worldBound.height),
        "Element does not have a defined size");

      var parent = FindElement(createTarget.Parent);
      var newElement = ((IMasonElement)element).Clone(_registry);
      newElement.name = "<Target>";
      newElement.style.visibility = Visibility.Hidden;
      newElement.style.width = 1;
      newElement.style.height = 1;

      if (parent.childCount <= createTarget.Index)
      {
        parent.Add(newElement);
      }
      else
      {
        parent.Insert((int)createTarget.Index, newElement);
      }
      
      _targets[createTarget.TargetName] = newElement;

      TweenUtils.Sequence("ShowTarget")
        .Insert(0, DOTween.To(() => newElement.style.height.value.value,
          x => newElement.style.height = x,
          element.worldBound.height,
          Seconds(createTarget.Animation.Duration)))
        .Insert(0, DOTween.To(() => newElement.style.width.value.value,
          x => newElement.style.width = x,
          element.worldBound.width,
          Seconds(createTarget.Animation.Duration)))
        .InsertCallback(Seconds(createTarget.Animation.Duration), () =>
        {
          newElement.style.visibility = Visibility.Visible;
        });
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
          return Errors.CheckNotNull(_registry.InputService.CurrentDragIndicator(), "Drag indicator not found");
        case ElementSelector.SelectorOneofCase.TargetElement:
          Errors.CheckState(_targets.ContainsKey(elementSelector.TargetElement),
            $"Target not found {elementSelector.TargetElement}");
          return _targets[elementSelector.TargetElement];
        default:
          throw new ArgumentOutOfRangeException();
      }
    }

    public bool ElementExists(ElementSelector elementSelector)
    {
      switch (elementSelector.SelectorCase)
      {
        case ElementSelector.SelectorOneofCase.ElementName:
          var elementName = Errors.CheckNotNull(elementSelector.ElementName);
          return _clones.ContainsKey(elementName) ||
                 _registry.DocumentService.RootVisualElement.Q(elementName) != null;
        case ElementSelector.SelectorOneofCase.DragIndicator:
          return _registry.InputService.CurrentDragIndicator() != null;
        default:
          return false;
      }
    }

    /// <summary>
    /// Invokes a callback immediately at time 0, or schedules it to run in the provided Sequence. We check this
    /// so that things like clones happen sequentially before other operations.
    /// </summary>
    void AddCallbackOrInvoke(Sequence sequence, TimeValue timeValue, TweenCallback action)
    {
      if (timeValue.Milliseconds == 0)
      {
        action();
      }
      else
      {
        sequence.InsertCallback(Seconds(timeValue), action);
      }
    }

    static float Seconds(TimeValue value) => value.Milliseconds / 1000f;
  }
}