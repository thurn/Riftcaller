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
using System.Diagnostics.CodeAnalysis;
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
    [SerializeField] Transform _userManaTooltipAnchor = null!;
    [SerializeField] Transform _opponentManaTooltipAnchor = null!;

    [SuppressMessage("ReSharper", "NotAccessedPositionalProperty.Local")]
    sealed record EffectKey(
      TutorialEffect.TutorialEffectTypeOneofCase EffectType,
      ArrowBubbleAnchor? Anchor = null);

    sealed class EffectData
    {
      readonly List<Sequence> _sequences = new();
      readonly List<GameObject> _gameObjects = new();
      readonly List<VisualElement> _elements = new();

      public void AddSequence(Sequence sequence)
      {
        _sequences.Add(sequence);
      }
      
      public void AddObject(GameObject gameObject)
      {
        _gameObjects.Add(gameObject);
      }

      public void AddElement(VisualElement element)
      {
        _elements.Add(element);
      }

      public void Merge(EffectData other)
      {
        _sequences.AddRange(other._sequences);
        _gameObjects.AddRange(other._gameObjects);
        _elements.AddRange(other._elements);
      }

      public void DestroyAll()
      {
        _sequences.ForEach(s => s.Kill());
        _sequences.Clear();
        _gameObjects.ForEach(Destroy);
        _gameObjects.Clear();
        _elements.ForEach(e => e.RemoveFromHierarchy());
        _elements.Clear();
      }
    }
    
    readonly Dictionary<EffectKey, EffectData> _effectMap = new();

    public void ClearTutorialEffects()
    {
      foreach (var data in _effectMap.Values)
      {
        data.DestroyAll();
      }
    }

    /// <summary>Displays the provided tutorial elements and clears all existing elements.</summary>
    public void SetTutorialEffects(IEnumerable<TutorialEffect> effects)
    {
      var newEffectMap = new Dictionary<EffectKey, EffectData>();
      
      foreach (var effect in effects)
      {
        switch (effect.TutorialEffectTypeCase)
        {
          case TutorialEffect.TutorialEffectTypeOneofCase.ArrowBubble:
            CreateArrowBubble(newEffectMap, effect.ArrowBubble);
            break;
          case TutorialEffect.TutorialEffectTypeOneofCase.ShowToast:
            CreateToast(newEffectMap, effect.ShowToast);
            break;
          default:
            throw new ArgumentOutOfRangeException();
        }
      }

      foreach (var (key, value) in newEffectMap)
      {
        CreateEffectDataIfNeeded(_effectMap, key);
        _effectMap[key].Merge(value);
      }
    }

    void CreateArrowBubble(Dictionary<EffectKey, EffectData> newEffectMap, ShowArrowBubble showBubble)
    {
      var key = new EffectKey(
        TutorialEffect.TutorialEffectTypeOneofCase.ArrowBubble,
        showBubble.Anchor);
      RemoveEffectIfExists(key);
      
      var bubble = NewArrowBubble(showBubble);
      bubble.transform.localScale = Vector3.zero;
      var showTime = DataUtils.ToSeconds(showBubble.IdleTimer, 0);
      var hideTime = DataUtils.ToSeconds(showBubble.HideTime, 0);
      var sequence = TweenUtils.Sequence("ShowArrowBubble")
        .Insert(showTime, bubble.transform.DOScale(Vector3.one, 0.3f));
      
      if (hideTime != 0)
      {
        sequence.Insert(showTime + hideTime, bubble.transform.DOScale(Vector3.zero, 0.3f));        
      }

      CreateEffectDataIfNeeded(newEffectMap, key);
      newEffectMap[key].AddSequence(sequence);
      newEffectMap[key].AddObject(bubble.gameObject);
    }
    
    ArrowBubble NewArrowBubble(ShowArrowBubble showBubble)
    {
      var component = ComponentUtils.Instantiate(_arrowBubblePrefab);
      component.ApplyStyle(showBubble);
      var parent = AnchorTransform(showBubble.Anchor);
      var anchorPosition = parent.position;
      
      Transform? offsetPosition = null!;
      if (_registry.RaidService.RaidActive)
      {
        offsetPosition = parent.Find("RaidAnchorPosition");
      }
      if (!offsetPosition)
      {
        offsetPosition = parent.Find("AnchorPosition");
      }
      if (offsetPosition)
      {
        anchorPosition.x += offsetPosition.localPosition.x;
        anchorPosition.z += offsetPosition.localPosition.z;
      }
      
      // Offset position to be bottom-left anchored
      anchorPosition.x -= (ArrowBubble.DefaultDeltaSize.x / 2.0f);
      anchorPosition.z -= (ArrowBubble.DefaultDeltaSize.y / 2.0f);
      anchorPosition.y = 5f;
      component.transform.position = anchorPosition;
      component.transform.localEulerAngles = _registry.MainCamera.transform.localEulerAngles;
      return component;
    }

    Transform AnchorTransform(ArrowBubbleAnchor anchor) => anchor.BubbleAnchorCase switch
    {
      ArrowBubbleAnchor.BubbleAnchorOneofCase.Player => 
        _registry.GameCharacterForPlayer(anchor.Player).transform,
      ArrowBubbleAnchor.BubbleAnchorOneofCase.Room => 
        _registry.ArenaService.FindRoom(anchor.Room).transform,
      ArrowBubbleAnchor.BubbleAnchorOneofCase.PlayerDeck => 
        _registry.DeckForPlayer(anchor.PlayerDeck).transform,
      ArrowBubbleAnchor.BubbleAnchorOneofCase.PlayerMana => 
        anchor.PlayerMana == PlayerName.User ? _userManaTooltipAnchor : _opponentManaTooltipAnchor,
      _ => throw new ArgumentOutOfRangeException(
        nameof(anchor.BubbleAnchorCase), anchor.BubbleAnchorCase, null)
    };

    void CreateToast(Dictionary<EffectKey, EffectData> newEffectMap, ShowToast showToast)
    {
      var key = new EffectKey(TutorialEffect.TutorialEffectTypeOneofCase.ShowToast);
      RemoveEffectIfExists(key);
      
      var toast = Mason.Render(_registry, showToast.Node);
      toast.style.position = Position.Absolute;
      toast.style.bottom = Screen.height;
      toast.style.left = 150 + Screen.safeArea.xMin;

      _registry.DocumentService.RootVisualElement.Insert(0, toast);
      var sequence = TweenUtils.Sequence("ShowToast");
      var showTime = DataUtils.ToSeconds(showToast.IdleTimer, 0);
      var hideTime = DataUtils.ToSeconds(showToast.HideTime, 0);

      sequence.InsertCallback(0, () =>
      {
        toast.style.top = -toast.worldBound.height;
        toast.style.bottom = new StyleLength(StyleKeyword.Null);
      })
        .Insert(showTime, DOTween.To(() => toast.style.top.value.value,
        y => toast.style.top = y,
        16,
        0.3f));

      if (hideTime != 0)
      {
        sequence.Insert(showTime + hideTime, HideToast(toast));
      }

      CreateEffectDataIfNeeded(newEffectMap, key);
      newEffectMap[key].AddSequence(sequence);
      newEffectMap[key].AddElement(toast);
    }

    void CreateEffectDataIfNeeded(Dictionary<EffectKey, EffectData> newEffectMap, EffectKey key)
    {
      if (!newEffectMap.ContainsKey(key))
      {
        newEffectMap[key] = new EffectData();
      }
    }

    void RemoveEffectIfExists(EffectKey key)
    {
      if (_effectMap.ContainsKey(key))
      {
        _effectMap[key].DestroyAll();
      }
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