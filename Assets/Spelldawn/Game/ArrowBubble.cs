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

using Spelldawn.Masonry;
using Spelldawn.Protos;
using TMPro;
using UnityEngine;
using UnityEngine.UI;

namespace Spelldawn.Game
{
  /// <summary>
  /// Represents a text box with an arrow, typically used as a speech bubble or help tooltip.
  /// </summary>
  public sealed class ArrowBubble : MonoBehaviour
  {
    public static readonly Vector3 DefaultDeltaSize = new Vector2(9.0f, 3.0f); 
    
    [SerializeField] TextMeshPro _text = null!;
    [SerializeField] SpriteRenderer _background = null!;
    
    public void ApplyStyle(ShowArrowBubble arrowBubble)
    {
      _text.text = arrowBubble.Text;
      _background.color = arrowBubble.Color != null ? Mason.ToUnityColor(arrowBubble.Color) : Color.white;
      _text.color = arrowBubble.FontColor != null ? Mason.ToUnityColor(arrowBubble.FontColor) : Color.black;
      var scale = (arrowBubble.Scale ?? 1.1f);
      _background.transform.localScale = scale * Vector3.one;
      _background.flipX = arrowBubble.ArrowCorner == ArrowBubbleCorner.BottomRight;
      _text.rectTransform.sizeDelta = scale * DefaultDeltaSize;
    }
  }
}