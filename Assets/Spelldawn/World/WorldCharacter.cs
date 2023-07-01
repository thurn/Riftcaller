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
using Spelldawn.Utils;
using UnityEngine;

namespace Spelldawn.World
{
  public sealed class WorldCharacter : MonoBehaviour
  {
    bool _initialized;
    [SerializeField] AnimatedCharacter _character = null!;
    WorldMap _worldMap = null!;
    float _moveSpeed;
    Action? _onArriveAtDestination;
    
    readonly Queue<Vector2> _targetPositions = new();

    public void Initialize(WorldMap worldMap)
    {
      Errors.CheckNotNull(_character);
      Errors.CheckNotNull(worldMap);
      _character.SetDirection(AnimatedCharacter.Direction.Right);
      _character.SetSpeed(0f);
      _worldMap = worldMap;
      _initialized = true;
    }

    public bool Moving => _targetPositions.Count > 0;

    void Update()
    {
      Errors.CheckState(_initialized, "WorldCharacter not initialized");
      
      if (_targetPositions.Count > 0)
      {
        var target = _targetPositions.Peek();
        
        var step =  _moveSpeed * Time.deltaTime;
        transform.position = Vector3.MoveTowards(transform.position, target, step);

        if (Vector3.Distance(transform.position, target) < 0.001f)
        {
          _targetPositions.Dequeue();
          if (_targetPositions.Count == 0)
          {
            _character.SetSpeed(0f);
            _onArriveAtDestination?.Invoke();
            _onArriveAtDestination = null;
          }
          else
          {
            SetDirectionForTarget(_targetPositions.Peek());
          }
        }            
      }

      var mapPosition = _worldMap.FromWorldPosition(transform.position);
      _character.SortingGroup.sortingOrder = _worldMap.SortOrderForTileId(new WorldMap.TileId(mapPosition, 10));
    }

    public void MoveOnPath(List<Vector3> positions, Action? onArriveAtDestination = null)
    {
      if (positions.Count > 0)
      {
        _targetPositions.Clear();
        _character.SetSpeed(0.5f);
        _moveSpeed = 3.0f;
        foreach (var p in positions)
        {
          _targetPositions.Enqueue(p);
        }
        
        SetDirectionForTarget(_targetPositions.Peek());
        _onArriveAtDestination = onArriveAtDestination;
      }
      else
      {
        onArriveAtDestination?.Invoke();
      }
    }

    void SetDirectionForTarget(Vector2 target)
    {
      var direction = (target - ((Vector2)transform.position)).normalized;
      if (Mathf.Abs(direction.x) > Mathf.Abs(direction.y))
      {
        _character.SetDirection(direction.x < 0 ? AnimatedCharacter.Direction.Left : AnimatedCharacter.Direction.Right);
      }
      else
      {
        _character.SetDirection(direction.y < 0 ? AnimatedCharacter.Direction.Down : AnimatedCharacter.Direction.Up);
      }
    }
  }
}