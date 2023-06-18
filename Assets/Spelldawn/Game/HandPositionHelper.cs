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

using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine;
using UnityEngine.Serialization;

namespace Spelldawn.Game
{
  public sealed class HandPositionHelper : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] Vector2 _offset;
    [SerializeField] PlayerName _owner;
    [SerializeField] float _cameraDistance;
    
    void Update()
    {
      var target = _registry.MainCamera.ScreenToWorldPoint(
        new Vector3(
          _offset.x + Screen.width / 2.0f, 
          _owner == PlayerName.User ? _offset.y : Screen.height - _offset.y,
          _cameraDistance));
      transform.position = target;
    }    
  }
}