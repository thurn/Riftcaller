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

namespace Spelldawn.World
{
  public sealed class WorldCamera : MonoBehaviour
  {
    [SerializeField] Camera _camera = null!;
    Vector3 _dragStartPosition;

    void Update () {
      if (Input.GetMouseButtonDown(0))
      {
        _dragStartPosition = new Vector3(Input.mousePosition.x, Input.mousePosition.y, -10);
        _dragStartPosition = _camera.ScreenToWorldPoint(_dragStartPosition);
        _dragStartPosition.z = transform.position.z;
      }
      else if (Input.GetMouseButton(0))
      {
        var mouseMove = new Vector3(Input.mousePosition.x, Input.mousePosition.y, -10);
        mouseMove = _camera.ScreenToWorldPoint(mouseMove);
        mouseMove.z = transform.position.z;
        transform.position -= mouseMove - _dragStartPosition;
      }
    }
  }
}