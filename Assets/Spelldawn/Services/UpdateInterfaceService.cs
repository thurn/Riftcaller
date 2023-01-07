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
using TimeValue = Spelldawn.Protos.TimeValue;

namespace Spelldawn.Services
{
  public sealed class UpdateInterfaceService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    public HashSet<VisualElement> HiddenForAnimation { get; } = new();

    public IEnumerator HandleUpdateInterface(UpdateInterfaceElementCommand command) =>
      command.InterfaceUpdateCase switch
      {
        UpdateInterfaceElementCommand.InterfaceUpdateOneofCase.AnimateToChildIndex => HandleAnimateToChildIndex(
          command.ElementName, command.AnimateToChildIndex),
        UpdateInterfaceElementCommand.InterfaceUpdateOneofCase.AnimateToElementPosition =>
          HandleAnimateToElementPosition(command.ElementName, command.AnimateToElementPosition),
        UpdateInterfaceElementCommand.InterfaceUpdateOneofCase.Destroy => HandleDestroy(command.ElementName),
        UpdateInterfaceElementCommand.InterfaceUpdateOneofCase.UpdateText => HandleUpdateText(command.ElementName,
          command.UpdateText),
        UpdateInterfaceElementCommand.InterfaceUpdateOneofCase.ClearChildren =>
          HandleClearChildren(command.ElementName),
        _ => throw new ArgumentOutOfRangeException()
      };

    IEnumerator HandleUpdateText(string elementName, UpdateText command)
    {
      Debug.Log($"HandleUpdateText");
      yield break;
    }

    IEnumerator HandleDestroy(string elementName)
    {
      var element = FindElement<VisualElement>(elementName);
      element.RemoveFromHierarchy();
      yield break;
    }

    IEnumerator HandleAnimateToElementPosition(string elementName, AnimateToElementPositionAndDestroy command)
    {
      var sourceElement = FindElement<VisualElement>(elementName);

      VisualElement target;
      if (string.IsNullOrEmpty(command.FallbackTargetElementName))
      {
        target = FindElement<VisualElement>(command.TargetElementName);
      }
      else
      {
        target = FindElementOptional<VisualElement>(command.TargetElementName) ??
                 FindElement<VisualElement>(command.FallbackTargetElementName);
      }

      yield return AnimateToPositionAndDestroy(sourceElement, target.worldBound, command.Animation, doNotClone: command.DoNotClone).WaitForCompletion();
    }

    IEnumerator HandleAnimateToChildIndex(string elementName, AnimateDraggableToChildIndex command)
    {
      var element = FindElement<Draggable>(elementName);
      var parent = FindElement<VisualElement>(command.ParentElementName);

      var newElement = Mason.Render(_registry, element.Node);
      if (parent.childCount <= command.Index)
      {
        parent.Add(newElement);
      }
      else
      {
        parent.Insert((int)command.Index, newElement);
      }

      var targetHeight = newElement.style.height;
      newElement.style.opacity = 0f;
      newElement.style.height = 0f;
      newElement.style.transitionProperty = new StyleList<StylePropertyName>(
        new List<StylePropertyName> { "height" });
      newElement.style.transitionDuration = new StyleList<UnityEngine.UIElements.TimeValue>(
        new List<UnityEngine.UIElements.TimeValue> { new(command.Duration.Milliseconds, TimeUnit.Millisecond) });
      HiddenForAnimation.Add(newElement);

      yield return new WaitForEndOfFrame();

      newElement.style.height = targetHeight;
      yield return AnimateToPositionAndDestroy(element, newElement.worldBound,
        new DestroyElementAnimation { Duration = command.Duration }, doNotClone: false, () =>
        {
          newElement.style.opacity = 1.0f;
          HiddenForAnimation.Remove(newElement);
        }).WaitForCompletion();
    }

    IEnumerator HandleClearChildren(string elementName)
    {
      var element = FindElement<VisualElement>(elementName);
      element.Clear();
      yield break;
    }

    public Sequence AnimateToPositionAndDestroy(
      VisualElement input,
      Rect targetBound,
      DestroyElementAnimation destroyAnimation,
      bool doNotClone = false,
      Action? onComplete = null)
    {
      Errors.CheckNotNull(input);
      Errors.CheckNotNull(destroyAnimation);
      
      var copy = doNotClone ? input : DetachCopy(input);
      // For shrink animations, we need to offset the target position based on the source element
      // size. This is because Unity calculates positions *before* applying scale transformations.
      var targetPosition = targetBound.position -
                           new Vector2(input.worldBound.width / 2.0f, input.worldBound.height / 2.0f) -
                           new Vector2(input.style.marginLeft.value.value, input.style.marginTop.value.value) +
                           new Vector2(targetBound.width / 2.0f, targetBound.height / 2.0f);

      var sequence = TweenUtils.Sequence("AnimateToPositionAndDestroy").Insert(0,
          DOTween.To(() => copy.style.left.value.value,
            x => copy.style.left = x,
            endValue: targetPosition.x, destroyAnimation.Duration.Milliseconds / 1000f))
        .Insert(0,
          DOTween.To(() => copy.style.top.value.value,
            y => copy.style.top = y,
            endValue: targetPosition.y, destroyAnimation.Duration.Milliseconds / 1000f));
      ApplyDestroyAnimation(sequence, destroyAnimation, copy);
      return sequence.AppendCallback(() =>
      {
        copy.RemoveFromHierarchy();
        onComplete?.Invoke();
      });
    }

    public static Sequence AnimateToZeroHeightAndDestroy(VisualElement element, TimeValue duration)
    {
      return TweenUtils.Sequence("AnimateToZeroHeightAndDestroy").Append(
          DOTween.To(() => element.style.height.value.value,
            height => element.style.height = height,
            endValue: 0,
            duration.Milliseconds / 1000f))
        .AppendCallback(element.RemoveFromHierarchy);
    }

    T FindElement<T>(string elementName) where T : VisualElement
    {
      var results = _registry.DocumentService.RootVisualElement.Query<T>(elementName).Build();
      Errors.CheckState(results.Count() == 1, $"Expected exactly 1 {elementName} but got {results.Count()}");
      return results.First();
    }

    T? FindElementOptional<T>(string elementName) where T : VisualElement
    {
      var results = _registry.DocumentService.RootVisualElement.Query<T>(elementName).Build();
      switch (results.Count())
      {
        case 0:
          return null;
        case 1:
          return results.First();
        default:
          throw new InvalidOperationException($"Found more than one element named {elementName}");
      }
    }

    void ApplyDestroyAnimation(Sequence sequence, DestroyElementAnimation destroyAnimation, VisualElement element)
    {
      foreach (var effect in destroyAnimation.Effects)
      {
        switch (effect)
        {
          case DestroyAnimationEffect.ShrinkHeight:
            sequence.Insert(0, DOTween.To(() => element.style.height.value.value,
              height => element.style.height = height,
              endValue: 0,
              destroyAnimation.Duration.Milliseconds / 1000f));
            break;
          case DestroyAnimationEffect.Shrink:
            sequence.Insert(0, DOTween.To(() => 1.0f,
              x => element.style.scale = new Scale(new Vector3(x, x, 1)),
              endValue: 0f,
              destroyAnimation.Duration.Milliseconds / 1000f));
            break;
          case DestroyAnimationEffect.FadeOut:
            sequence.Insert(0, DOTween.To(() => element.style.opacity.value,
              opacity => element.style.opacity = opacity,
              endValue: 0,
              destroyAnimation.Duration.Milliseconds / 1000f));
            break;
        }
      }
    }

    VisualElement DetachCopy(VisualElement input)
    {
      var element = ((IMasonElement)input).Clone(_registry);
      input.style.visibility = Visibility.Hidden;
      var initialPosition = input.worldBound.position;
      element.name = "<Animating>";
      element.style.left = initialPosition.x;
      element.style.top = initialPosition.y;
      element.style.position = Position.Absolute;
      _registry.DocumentService.RootVisualElement.Add(element);
      return element;
    }
  }
}