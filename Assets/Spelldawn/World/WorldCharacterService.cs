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
  public sealed class WorldCharacterService : MonoBehaviour
  {
    [SerializeField] WorldCharacter _characterPrefab = null!;
    [SerializeField] WorldMap _worldMap = null!;
    WorldCharacter _hero = null!;

    public WorldCharacter Hero => _hero;

    public void Start()
    {
      _hero = ComponentUtils.Instantiate(_characterPrefab);
      _hero.transform.position = new Vector3(0, 0, 0);
      _hero.Initialize(_worldMap);
    }

    public void MoveHero(List<Vector3> path, Action? onArrive = null)
    {
      _hero.MoveOnPath(path, onArrive);
    }

    public Vector3Int CurrentHeroPosition()
    {
      var result = _worldMap.FromWorldPosition(_hero.transform.position);
      return new Vector3Int(result.X, result.Y, 0);
    }
  }
}