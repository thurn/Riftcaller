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
using UnityEngine;

namespace Spelldawn.World
{
  public sealed class WorldCharacter : MonoBehaviour
  {
    public const float CharacterOffset = 2.25f;
    static readonly int SpeedParam = Animator.StringToHash("Speed");
    static readonly int DirectionParam = Animator.StringToHash("Direction");
    
    const float AnimatorUp = 0f;
    const float AnimatorSide = 1f;
    const float AnimatorDown = 2f;

    [SerializeField] Animator _animator = null!;
    [SerializeField] GameObject _down = null!;
    [SerializeField] GameObject _side = null!;
    [SerializeField] GameObject _up = null!;
    float _moveSpeed;
    
    readonly Queue<Vector2> _targetPositions = new();

    enum Direction
    {
      Up,
      Down,
      Left,
      Right
    }

    public void Initialize()
    {
      SetDirection(Direction.Right);
      _animator.SetFloat(SpeedParam, 0f);      
    }

    public bool Moving => _targetPositions.Count > 0;

    void Update()
    {
      if (Input.GetKeyDown(KeyCode.W))
      {
        SetDirection(Direction.Up);
      }
      
      if (Input.GetKeyDown(KeyCode.A))
      {
        SetDirection(Direction.Left);
      }
      
      if (Input.GetKeyDown(KeyCode.S))
      {
        SetDirection(Direction.Down);
      }
      
      if (Input.GetKeyDown(KeyCode.D))
      {
        SetDirection(Direction.Right);
      }

      if (Input.GetKeyDown(KeyCode.Q))
      {
        _animator.SetFloat(SpeedParam, 0.5f);
      }

      if (Input.GetKeyDown(KeyCode.E))
      {
        _animator.SetFloat(SpeedParam, 0f);
      }

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
            _animator.SetFloat(SpeedParam, 0f);
          }
          else
          {
            SetDirectionForTarget(_targetPositions.Peek());
          }
        }            
      }
    }

    public void MoveOnPath(List<Vector2> positions)
    {
      if (positions.Count > 0)
      {
        _targetPositions.Clear();
        _animator.SetFloat(SpeedParam, 0.5f);
        _moveSpeed = 3.0f;
        foreach (var p in positions)
        {
          _targetPositions.Enqueue(p);
        }
        
        SetDirectionForTarget(_targetPositions.Peek());
      }
    }

    void SetDirectionForTarget(Vector2 target)
    {
      var direction = (target - ((Vector2)transform.position)).normalized;
      if (Mathf.Abs(direction.x) > Mathf.Abs(direction.y))
      {
        SetDirection(direction.x < 0 ? Direction.Left : Direction.Right);
      }
      else
      {
        SetDirection(direction.y < 0 ? Direction.Down : Direction.Up);
      }
    }

    void SetDirection(Direction direction)
    {
      _down.SetActive(false);
      _up.SetActive(false);
      _side.SetActive(false);

      switch (direction)
      {
        case Direction.Up:
          _up.SetActive(true);
          _animator.SetFloat(DirectionParam, AnimatorUp);
          break;
        case Direction.Down:
          _down.SetActive(true);
          _animator.SetFloat(DirectionParam, AnimatorDown);          
          break;
        case Direction.Left:
          _side.SetActive(true);
          _animator.SetFloat(DirectionParam, AnimatorSide);
          var s1 = _side.transform.localScale;
          s1.x = Mathf.Abs(s1.x) * -1;
          _side.transform.localScale = s1;          
          break;
        case Direction.Right:
          _side.SetActive(true);
          _animator.SetFloat(DirectionParam, AnimatorSide);
          var s2 = _side.transform.localScale;
          s2.x = Mathf.Abs(s2.x);
          _side.transform.localScale = s2;             
          break;
        default:
          throw new ArgumentOutOfRangeException(nameof(direction), direction, null);
      }
    }
  }
}