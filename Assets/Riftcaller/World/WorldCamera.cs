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

using Riftcaller.Services;
using UnityEngine;

namespace Riftcaller.World
{
  public sealed class WorldCamera : MonoBehaviour
  {
    [SerializeField] Camera _camera = null!;
    [SerializeField] WorldMap _worldMap = null!;
    [SerializeField] Registry _registry = null!;
    [SerializeField] float _panSpeed;
    Vector3? _dragStartScreenPosition;
    Vector3 _dragStartPosition;
    bool _initialized;

    void Update() {
      if (_registry.DocumentService.MouseOverScreenElement() || _registry.DocumentService.IsAnyPanelOpen())
      {
        return;
      }
      
      if (Input.GetMouseButtonDown(0))
      {
        // Unity sends MouseButtonDown again on release for mobile devices?!
        _dragStartScreenPosition ??= Input.mousePosition;
        _dragStartPosition = MousePosition();
      }
      else if (Input.GetMouseButton(0))
      {
        var mouseMove = MousePosition();
        transform.position -= mouseMove - _dragStartPosition;
      }
      else if (Input.GetMouseButtonUp(0) && _dragStartScreenPosition is {} startPosition)
      {
        var mouseUp = Input.mousePosition;
        if (Vector2.Distance(startPosition, mouseUp) < 10f)
        {
          _worldMap.OnClick(MousePosition());
        }

        _dragStartScreenPosition = null;
      }
      else if (_registry.CharacterService.Hero && _registry.CharacterService.Hero!.Moving)
      {
        var position = _registry.CharacterService.Hero.gameObject.transform.position;
        var step =  _panSpeed * Time.deltaTime;
        var target = new Vector3(position.x, position.y, transform.position.z);
        transform.position = Vector3.MoveTowards(transform.position, target, step);
      }
    }

    Vector3 MousePosition()
    {
      var position = new Vector3(Input.mousePosition.x, Input.mousePosition.y, -10);
      position = _camera.ScreenToWorldPoint(position);
      position.z = transform.position.z;
      return position;
    }
  }
}