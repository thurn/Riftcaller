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

using System;
using Riftcaller.Protos;
using UnityEngine;
using UnityEngine.Rendering;

namespace Riftcaller.World
{
  public sealed class AnimatedCharacter : MonoBehaviour
  {
    static readonly int SpeedParam = Animator.StringToHash("Speed");
    static readonly int DirectionParam = Animator.StringToHash("Direction");
    
    const float AnimatorUp = 0f;
    const float AnimatorSide = 1f;
    const float AnimatorDown = 2f;

    [SerializeField] Animator _animator = null!;
    [SerializeField] SortingGroup _sortingGroup = null!;
    [SerializeField] GameObject _down = null!;
    [SerializeField] GameObject _side = null!;
    [SerializeField] GameObject _up = null!;

    public SortingGroup SortingGroup => _sortingGroup;

    public enum Direction
    {
      Up,
      Down,
      Left,
      Right
    }

    public void SetSpeed(float speed)
    {
      _animator.SetFloat(SpeedParam, speed);
    }
    
    public void SetFacingDirection(GameCharacterFacingDirection direction)
    {
      switch (direction)
      {
        case GameCharacterFacingDirection.Up:
          SetDirection(AnimatedCharacter.Direction.Up);
          break;
        case GameCharacterFacingDirection.Down:
          SetDirection(AnimatedCharacter.Direction.Down);          
          break;
        case GameCharacterFacingDirection.Left:
          SetDirection(AnimatedCharacter.Direction.Left);          
          break;
        case GameCharacterFacingDirection.Right:
          SetDirection(AnimatedCharacter.Direction.Right);          
          break;
        case GameCharacterFacingDirection.Unspecified:
        default:
          throw new ArgumentOutOfRangeException(nameof(direction), direction, null);
      }
    }    
    
    public void SetDirection(Direction direction)
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
        case Direction.Right:
          _side.SetActive(true);
          _animator.SetFloat(DirectionParam, AnimatorSide);
          var s1 = _side.transform.localScale;
          s1.x = Mathf.Abs(s1.x) * -1;
          _side.transform.localScale = s1;          
          break;
        case Direction.Left:
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