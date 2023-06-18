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
using Spelldawn.Services;
using UnityEngine;

namespace Spelldawn.Game
{
  public sealed class ViewportAnchor : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] Vector2 _offsetPercent;
    [SerializeField] float _zPosition;
    
    void Update()
    {
      var target = _registry.MainCamera.ViewportToWorldPoint(
        new Vector3(_offsetPercent.x / 100f, _offsetPercent.y / 100f, _zPosition));
      transform.position = target;
    }
  }
}