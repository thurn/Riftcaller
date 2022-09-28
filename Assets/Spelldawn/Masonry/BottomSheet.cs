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

using System.Collections;
using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.UIElements;

namespace Spelldawn.Masonry
{
  public sealed class BottomSheet : VisualElement
  {
    const float AnimationDurationSeconds = 0.3f;
    readonly VisualElement _backgroundOverlay;
    readonly VisualElement _sheet;
    readonly Registry _registry;
    readonly List<InterfacePanelAddress> _stack = new();
    bool _isOpen;
    bool _isAnimating;
    VisualElement _content = new NodeVisualElement();

    public BottomSheet(Registry registry)
    {
      _registry = registry;
      name = "BottomSheetContainer";
      style.position = Position.Absolute;
      style.top = 0;
      style.right = 0;
      style.bottom = 0;
      style.left = 0;
      pickingMode = PickingMode.Ignore;

      _backgroundOverlay = new VisualElement
      {
        name = "BottomSheetOverlay",
        style =
        {
          opacity = 0,
          backgroundColor = Color.black,
          position = Position.Absolute,
          top = 0,
          right = 0,
          bottom = 0,
          left = 0
        },
        pickingMode = PickingMode.Ignore
      };

      var safeArea = registry.DocumentService.GetSafeArea();
      _sheet = new VisualElement
      {
        name = "BottomSheet",
        style =
        {
          position = Position.Absolute,
          top = Length.Percent(100f),
          right = safeArea.Right.Value + 16,
          bottom = safeArea.Bottom.Value + 0,
          left = safeArea.Left.Value + 16,
          backgroundColor = Color.gray,
          borderTopLeftRadius = 24,
          borderTopRightRadius = 24,
          justifyContent = Justify.Center,
          alignItems = Align.Center,
          height = Length.Percent(95)
        }
      };

      Add(_backgroundOverlay);
      _sheet.Add(_content);
      Add(_sheet);
    }

    /// <summary>
    /// Opens the bottom sheet to display the panel with the provided address, closing any existing sheet.
    /// </summary>
    public IEnumerator OpenWithAddress(InterfacePanelAddress address)
    {
      if (_isOpen)
      {
        var last = _stack.LastOrDefault();
        if (last != null && last.Equals(address))
        {
          // If we're attempting to open an identical sheet, do nothing
          yield break;
        }
        else
        {
          yield return AnimateClose();
        }
      }

      _stack.Clear();
      _stack.Add(address);
      yield return AnimateOpen();
    }

    /// <summary>Closes the bottom sheet.</summary>
    public IEnumerator Close()
    {
      if (_isOpen)
      {
        yield return AnimateClose();
      }
      _stack.Clear();
    }

    /// <summary>Pushes a new page onto this bottom sheet</summary>
    public IEnumerator PushAddress(InterfacePanelAddress address)
    {
      yield break;
    }

    /// <summary>Removes the specified page from this bottom sheet.</summary>
    public IEnumerator PopAddress(InterfacePanelAddress address)
    {
      yield break;
    }

    public void RefreshPanels()
    {
      var address = _stack.LastOrDefault();
      if (address == null || !_registry.DocumentService.PanelCache.ContainsKey(address))
      {
        ClearContent();
      }
      else
      {
        var result = Reconciler.Update(_registry, _registry.DocumentService.PanelCache[address], _content);
        if (result != null)
        {
          SetContent(result);
        }         
      }
    }

    void SetContent(VisualElement content)
    {
      _content = content;
      _sheet.Clear();
      _sheet.Add(_content);
    }

    void ClearContent()
    {
      SetContent(new NodeVisualElement());
    }

    IEnumerator AnimateOpen()
    {
      if (!_isOpen && !_isAnimating)
      {
        _isAnimating = true;
        _sheet.style.top = Length.Percent(100f);
        yield return TweenUtils.Sequence("OpenBottomSheet")
          .Insert(0,
            DOTween.To(
              () => _backgroundOverlay.style.opacity.value,
              x => _backgroundOverlay.style.opacity = x,
              0.75f,
              AnimationDurationSeconds).SetEase(Ease.OutCirc))
          .Insert(0,
            DOTween.To(
              () => _sheet.style.top.value.value,
              x => _sheet.style.top = Length.Percent(x),
              5f,
              AnimationDurationSeconds).SetEase(Ease.OutCirc))
          .AppendCallback(() =>
          {
            _isAnimating = false;
            _isOpen = true;
          }).WaitForCompletion();
      }
    }

    IEnumerator AnimateClose()
    {
      if (_isOpen && !_isAnimating)
      {
        _isAnimating = true;
        _sheet.style.top = Length.Percent(5f);
        yield return TweenUtils.Sequence("CloseBottomSheet")
          .Insert(0,
            DOTween.To(
              () => _backgroundOverlay.style.opacity.value,
              x => _backgroundOverlay.style.opacity = x,
              0f,
              AnimationDurationSeconds).SetEase(Ease.OutCirc))
          .Insert(0,
            DOTween.To(
              () => _sheet.style.top.value.value,
              x => _sheet.style.top = Length.Percent(x),
              100f,
              AnimationDurationSeconds).SetEase(Ease.OutCirc))
          .AppendCallback(() =>
          {
            _isAnimating = false;
            _isOpen = false;
          }).WaitForCompletion();
      }
    }
  }
}