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

using Spelldawn.Services;
using UnityEngine;

namespace Spelldawn.Game
{
  public sealed class ViewportAnchor2 : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] Vector2 _offset;
    [SerializeField] float _zPosition;
    [SerializeField] TextAnchor _anchorPosition;
    void Update()
    {
      var point = _anchorPosition switch
      {
        TextAnchor.LowerLeft => 
          new Vector3(_offset.x + Screen.safeArea.xMin, _offset.y + Screen.safeArea.yMin, _zPosition),
        TextAnchor.LowerRight => 
          new Vector3(Screen.safeArea.xMax - _offset.x, _offset.y + Screen.safeArea.yMin, _zPosition),
        TextAnchor.UpperLeft => 
          new Vector3(_offset.x + Screen.safeArea.xMin, Screen.safeArea.yMax - _offset.y, _zPosition),
        TextAnchor.UpperRight => 
          new Vector3(Screen.safeArea.xMax - _offset.x, Screen.safeArea.yMax - _offset.y, _zPosition),          
      };
      var target = _registry.MainCamera.ScreenToWorldPoint(point);
      transform.position = target;
    }    
  }
}