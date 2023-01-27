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
using DG.Tweening;
using Spelldawn.Game;
using Spelldawn.Masonry;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.UIElements;

namespace Spelldawn.Services
{
  public sealed class TutorialService: MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] ArrowBubble _arrowBubblePrefab = null!;
    
    readonly List<GameObject> _effects = new();
    readonly List<Sequence> _sequences = new();
    readonly List<VisualElement> _toasts = new();
    VisualElement? _activeToast;

    public void ClearTutorialEffects()
    {
      _sequences.ForEach(s => s.Kill());
      _sequences.Clear();
      _effects.ForEach(Destroy);
      _effects.Clear();

      _toasts.ForEach(toast => HideToast(toast));
      _activeToast = null;
      _toasts.Clear();
    }
    
    /// <summary>Displays the provided tutorial elements and clears all existing elements.</summary>
    public void SetTutorialEffects(IEnumerable<TutorialEffect> effects)
    {
      ClearTutorialEffects();
      
      foreach (var effect in effects)
      {
        switch (effect.TutorialEffectTypeCase)
        {
          case TutorialEffect.TutorialEffectTypeOneofCase.ArrowBubble:
            _sequences.Add(AnimateArrowBubble(effect.ArrowBubble));
            break;
          case TutorialEffect.TutorialEffectTypeOneofCase.ShowToast:
            _sequences.Add(AnimateToast(effect.ShowToast));
            break;
          default:
            throw new ArgumentOutOfRangeException();
        }
      }
    }

    Sequence AnimateArrowBubble(ShowArrowBubble showBubble)
    {
      var bubble = CreateArrowBubble(showBubble);
      bubble.transform.localScale = Vector3.zero;
      var showTime = DataUtils.ToSeconds(showBubble.IdleTimer, 0);
      var hideTime = DataUtils.ToSeconds(showBubble.HideTime, 0);
      var sequence = TweenUtils.Sequence("ShowArrowBubble")
        .Insert(showTime, bubble.transform.DOScale(Vector3.one, 0.3f));
      
      if (hideTime != 0)
      {
        sequence.Insert(showTime + hideTime, bubble.transform.DOScale(Vector3.zero, 0.3f));        
      }

      return sequence;
    }
    
    ArrowBubble CreateArrowBubble(ShowArrowBubble showBubble)
    {
      var component = ComponentUtils.Instantiate(_arrowBubblePrefab);
      component.ApplyStyle(showBubble);
      var anchorPosition = AnchorTransform(showBubble).position + AnchorOffset(showBubble);
      
      // Offset position to be bottom-left anchored
      anchorPosition.x -= (ArrowBubble.DefaultDeltaSize.x / 2.0f);
      anchorPosition.z -= (ArrowBubble.DefaultDeltaSize.y / 2.0f);
      anchorPosition.y = 5f;
      component.transform.position = anchorPosition;
      component.transform.localEulerAngles = _registry.MainCamera.transform.localEulerAngles;
      _effects.Add(component.gameObject);
      return component;
    }

    Transform AnchorTransform(ShowArrowBubble showBubble) => showBubble.ArrowBubbleAnchorCase switch
    {
      ShowArrowBubble.ArrowBubbleAnchorOneofCase.Player => 
        _registry.LeaderCardForPlayer(showBubble.Player).transform,
      ShowArrowBubble.ArrowBubbleAnchorOneofCase.Room => 
        _registry.ArenaService.FindRoom(showBubble.Room).transform,
      ShowArrowBubble.ArrowBubbleAnchorOneofCase.PlayerDeck => 
        _registry.DeckForPlayer(showBubble.PlayerDeck).transform,
      ShowArrowBubble.ArrowBubbleAnchorOneofCase.PlayerMana => 
        _registry.ManaDisplayForPlayer(showBubble.PlayerMana).transform,
      _ => throw new ArgumentOutOfRangeException(
        nameof(showBubble.ArrowBubbleAnchorCase), showBubble.ArrowBubbleAnchorCase, null)
    };
    
    Vector3 AnchorOffset(ShowArrowBubble showBubble) => showBubble.ArrowBubbleAnchorCase switch
    {
      ShowArrowBubble.ArrowBubbleAnchorOneofCase.Player => 
        showBubble.Player == PlayerName.User ? new Vector3(-0.5f, 0f, -2f) : new Vector3(-1f, 0f, -1f),
      ShowArrowBubble.ArrowBubbleAnchorOneofCase.Room => new Vector3(3f, 0, 0),
      ShowArrowBubble.ArrowBubbleAnchorOneofCase.PlayerMana => new Vector3(5f, 0, -1.5f),
      ShowArrowBubble.ArrowBubbleAnchorOneofCase.PlayerDeck => new Vector3(-4f, 0, -2.5f),
      _ => Vector3.zero
    };

    Sequence AnimateToast(ShowToast showToast)
    {
      var toast = Mason.Render(_registry, showToast.Node);
      toast.style.position = Position.Absolute;
      toast.style.bottom = Screen.height;
      toast.style.left = 150 + Screen.safeArea.xMin;

      _registry.DocumentService.RootVisualElement.Add(toast);
      var sequence = TweenUtils.Sequence("ShowToast");
      var showTime = DataUtils.ToSeconds(showToast.IdleTimer, 0);
      var hideTime = DataUtils.ToSeconds(showToast.HideTime, 0);

      sequence.InsertCallback(0, () =>
      {
        toast.style.top = -toast.worldBound.height;
        toast.style.bottom = new StyleLength(StyleKeyword.Null);
      }).InsertCallback(Mathf.Max(0f, showTime - 0.3f), () =>
      {
        if (_activeToast != null)
        {
          HideToast(_activeToast);
        }
        _activeToast = toast;
      }).Insert(showTime, DOTween.To(() => toast.style.top.value.value,
        y => toast.style.top = y,
        16,
        0.3f));

      if (hideTime != 0)
      {
        sequence.Insert(showTime + hideTime, HideToast(toast));
      }

      _toasts.Add(toast);
      return sequence;
    }

    static Sequence HideToast(VisualElement toast)
    {
      return TweenUtils.Sequence("HideToast").Insert(0,
        DOTween.To(() => toast.style.top.value.value,
          y => toast.style.top = y,
          -toast.worldBound.height,
          0.3f)).AppendCallback(toast.RemoveFromHierarchy);
    }
  }
}