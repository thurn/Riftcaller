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

using UnityEngine;

namespace Spelldawn.Game
{
  public sealed class GameCamera : MonoBehaviour
  {
    [SerializeField] float _fieldOfView = 100.0f;
    [SerializeField] Camera _mainCamera = null!;

    void Update() {
      var safeAreaSize = Screen.safeArea.xMin + (Screen.width - Screen.safeArea.xMax);
      var targetFieldOfView = safeAreaSize > 150 ? _fieldOfView + 5 : _fieldOfView;
      var halfWidth = Mathf.Tan(0.5f * targetFieldOfView * Mathf.Deg2Rad);
      var halfHeight = halfWidth * Screen.height / Screen.width;
      var verticalFoV = 2.0f * Mathf.Atan(halfHeight) * Mathf.Rad2Deg;
      _mainCamera.fieldOfView = verticalFoV;
    }
  }
}