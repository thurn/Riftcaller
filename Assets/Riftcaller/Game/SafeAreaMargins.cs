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

using UnityEngine;

namespace Riftcaller.Game
{
  /// <summary>
  /// Removes visual margin from canvas UI elements when that margin is already
  /// provided by the device safe area.
  /// </summary>
  [RequireComponent(typeof(RectTransform))]
  public sealed class SafeAreaMargins : MonoBehaviour
  {
    [SerializeField] bool _removeXMin;
    [SerializeField] bool _removeXMax;
    [SerializeField] bool _removeYMin;
    [SerializeField] bool _removeYMax;

    void Start()
    {
      var rectTransform = GetComponent<RectTransform>();
      if (_removeXMin && Screen.safeArea.xMin > 0)
      {
        rectTransform.anchoredPosition = new Vector2(0f, rectTransform.anchoredPosition.y);
      }
      
      if (_removeYMin && Screen.safeArea.yMin > 0)
      {
        rectTransform.anchoredPosition = new Vector2(rectTransform.anchoredPosition.x, 0f);
      }
      
      if (_removeXMax && Screen.safeArea.xMax < Screen.width)
      {
        rectTransform.anchoredPosition = new Vector2(0f, rectTransform.anchoredPosition.y);
      }
      
      if (_removeYMax && Screen.safeArea.yMax < Screen.height)
      {
        rectTransform.anchoredPosition = new Vector2(rectTransform.anchoredPosition.x, 0f);
      }      
    }
  }
}