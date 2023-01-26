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
using Spelldawn.Game;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;

namespace Spelldawn.Services
{
  public sealed class TutorialService: MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] ArrowBubble _arrowBubblePrefab = null!;
    readonly List<GameObject> _effects = new();

    /// <summary>Displays the provided tutorial elements and clears all existing elements.</summary>
    public void SetTutorialEffects(IEnumerable<TutorialEffect> effects)
    {
      _effects.ForEach(Destroy);
      _effects.Clear();

      foreach (var effect in effects)
      {
        switch (effect.TutorialEffectTypeCase)
        {
          case TutorialEffect.TutorialEffectTypeOneofCase.ArrowBubble:
            DisplayArrowBubble(effect.ArrowBubble);
            break;
          default:
            throw new ArgumentOutOfRangeException();
        }
      }
    }

    void DisplayArrowBubble(ShowArrowBubble showBubble)
    {
      var component = ComponentUtils.Instantiate(_arrowBubblePrefab);
      component.ApplyStyle(showBubble);
      var anchorPosition = AnchorTransform(showBubble).position + AnchorOffset(showBubble);
      
      // Offset position to be bottom-left anchored
      anchorPosition.x -= (ArrowBubble.DefaultDeltaSize.x / 2.0f);
      anchorPosition.z -= (ArrowBubble.DefaultDeltaSize.y / 2.0f);
      anchorPosition.y = 0.5f;
      component.transform.position = anchorPosition;
      component.transform.localEulerAngles = _registry.MainCamera.transform.localEulerAngles;
      Debug.Log($"DisplayArrowBubble: {showBubble.Text}");
      _effects.Add(component.gameObject);
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
      ShowArrowBubble.ArrowBubbleAnchorOneofCase.Player => new Vector3(0f, 0f, -2f),
      _ => Vector3.zero
    };    
  }
}