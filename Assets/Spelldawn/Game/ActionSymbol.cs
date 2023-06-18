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

using DG.Tweening;
using Spelldawn.Utils;
using TMPro;
using UnityEngine;

namespace Spelldawn.Game
{
  public sealed class ActionSymbol : MonoBehaviour
  {
    [SerializeField] TextMeshProUGUI _text = null!;
    [SerializeField] Material _activeMaterial = null!;
    [SerializeField] Material _inactiveMaterial = null!;
    bool _filled = true;
    
    public bool IsAnimating { get; private set; } 

    public void SetFilled(bool filled)
    {
      if (filled)
      {
        gameObject.SetActive(true);
      }
      
      switch (filled)
      {
        case true when !_filled:
          IsAnimating = true;
          TweenUtils.Sequence("RotateActionSymbol")
            .Insert(0f, GetComponent<RectTransform>().DORotate(new Vector3(0f, 0f, 0.0f), 0.5f))
            .InsertCallback(0.25f, () => _text.fontMaterial = _activeMaterial)
            .AppendCallback(() => IsAnimating = false);
          break;
        case false when _filled:
          IsAnimating = true;
          TweenUtils.Sequence("RotateActionSymbol")
            .Insert(0f, GetComponent<RectTransform>().DORotate(new Vector3(0f, 0f, 180.0f), 0.5f))
            .InsertCallback(0.25f, () => _text.fontMaterial = _inactiveMaterial)
            .AppendCallback(() => IsAnimating = false);
          break;
      }

      _filled = filled;
    }

    public void SetFontMaterial(Material material)
    {
      _text.fontMaterial = material;
      _activeMaterial = material;
      var inactive = Instantiate(material);
      inactive.SetColor(ShaderUtilities.ID_FaceColor, Color.black);
      _inactiveMaterial = inactive;
    }
  }
}