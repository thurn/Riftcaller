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
    readonly Registry _registry;
    readonly VisualElement _backgroundOverlay;
    readonly VisualElement _sheet;
    VisualElement _contentContainer;
    VisualElement _content = new NodeVisualElement();
    InterfacePanelAddress? _currentAddress;
    bool _isOpen;
    bool _isAnimating;    

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
          height = Length.Percent(95),
          overflow = Overflow.Hidden
        }
      };
      
      _contentContainer = ContentContainer();
      Add(_backgroundOverlay);
      _contentContainer.Add(_content);
      _sheet.Add(_contentContainer);
      Add(_sheet);
    }

    static VisualElement ContentContainer()
    {
      return new VisualElement
      {
        name = "ContentContainer",
        style =
        {
          position = Position.Absolute,
          top = 0,
          right = 0,
          bottom = 0,
          left = 0,
          justifyContent = Justify.Center,
          alignItems = Align.Center,
          overflow = Overflow.Hidden
        }
      };
    }

    /// <summary>
    /// Opens the bottom sheet to display the panel with the provided address, closing any existing sheet.
    /// </summary>
    public IEnumerator OpenWithAddress(InterfacePanelAddress address)
    {
      if (_isOpen)
      {
        if (_currentAddress != null && _currentAddress.Equals(address))
        {
          // If we're attempting to open an identical sheet, do nothing
          yield break;
        }
        else
        {
          yield return AnimateClose();
        }
      }
      
      _currentAddress = address;
      RefreshPanels();
      yield return AnimateOpen();
    }

    /// <summary>Closes the bottom sheet.</summary>
    public IEnumerator Close()
    {
      if (_isOpen)
      {
        yield return AnimateClose();
      }

      _currentAddress = null;
      RefreshPanels();
    }

    /// <summary>Pushes a new page onto this bottom sheet</summary>
    public IEnumerator PushAddress(InterfacePanelAddress address)
    {
      if (!_isOpen || _currentAddress == null)
      {
        yield return OpenWithAddress(address);
      }
      else if (!_currentAddress.Equals(address))
      {
        yield return AnimateSlide(address, pop: false);
      }
    }

    /// <summary>Removes the current page from this bottom sheet and displays 'address' as the *new* content</summary>
    public IEnumerator PopToAddress(InterfacePanelAddress address)
    {
      if (!_isOpen || _currentAddress == null)
      {
        yield return OpenWithAddress(address);
      }
      else if (!_currentAddress.Equals(address))
      {
        yield return AnimateSlide(address, pop: true);
      }
    }

    public void RefreshPanels()
    {
      if (_currentAddress == null || !_registry.DocumentService.PanelCache.ContainsKey(_currentAddress))
      {
        _content.style.display = DisplayStyle.None;
      }
      else
      {
        _content.style.display = DisplayStyle.Flex;
        var node = _registry.DocumentService.PanelCache[_currentAddress].Node;
        var result = Reconciler.Update(_registry, node, _content);
        if (result != null)
        {
          SetContent(result);
        }         
      }
    }

    void SetContent(VisualElement content)
    {
      _content = content;
      _contentContainer.Clear();
      _contentContainer.Add(_content);
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
              AnimationDurationSeconds * TweenUtils.GlobalAnimationMultiplier).SetEase(Ease.OutCirc))
          .Insert(0,
            DOTween.To(
              () => _sheet.style.top.value.value,
              x => _sheet.style.top = Length.Percent(x),
              5f,
              AnimationDurationSeconds * TweenUtils.GlobalAnimationMultiplier).SetEase(Ease.OutCirc))
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
              AnimationDurationSeconds * TweenUtils.GlobalAnimationMultiplier).SetEase(Ease.OutCirc))
          .Insert(0,
            DOTween.To(
              () => _sheet.style.top.value.value,
              x => _sheet.style.top = Length.Percent(x),
              100f,
              AnimationDurationSeconds * TweenUtils.GlobalAnimationMultiplier).SetEase(Ease.OutCirc))
          .AppendCallback(() =>
          {
            _isAnimating = false;
            _isOpen = false;
          }).WaitForCompletion();
      }
    }
    
    IEnumerator AnimateSlide(InterfacePanelAddress address, bool pop)
    {
      if (_isOpen && !_isAnimating)
      {
        var oldContainer = _contentContainer;
        oldContainer.name = "OldContentContainer";
        _content = new NodeVisualElement();
        _contentContainer = ContentContainer();
        _contentContainer.Add(_content);
        _sheet.Add(_contentContainer);
        _currentAddress = address;
        
        _isAnimating = true;
        _contentContainer.style.translate = new Translate(Length.Percent(pop ? -100 : 100), Length.Percent(0), 0);
        oldContainer.style.translate = new Translate(Length.Percent(0), Length.Percent(0), 0);
        RefreshPanels();
        
        yield return TweenUtils.Sequence("PushBottomSheet")
          .Insert(0,
            DOTween.To(
              () => _contentContainer.style.translate.value.x.value,
              x => _contentContainer.style.translate = new Translate(Length.Percent(x), Length.Percent(0), 0),
              0f,
              AnimationDurationSeconds * TweenUtils.GlobalAnimationMultiplier).SetEase(Ease.OutCirc))
          .Insert(0,
            DOTween.To(
              () => oldContainer.style.translate.value.x.value,
              x => oldContainer.style.translate = new Translate(Length.Percent(x), Length.Percent(0), 0),
              pop ? 100 : -100,
              AnimationDurationSeconds * TweenUtils.GlobalAnimationMultiplier).SetEase(Ease.OutCirc))
          .AppendCallback(() =>
          {
            _isAnimating = false;
            oldContainer.RemoveFromHierarchy();
          }).WaitForCompletion();
      }
    }
  }
}