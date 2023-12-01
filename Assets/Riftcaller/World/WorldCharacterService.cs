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
using System.Collections.Generic;
using Riftcaller.Protos;
using Riftcaller.Services;
using Riftcaller.Utils;
using UnityEngine;

namespace Riftcaller.World
{
  public sealed class WorldCharacterService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] WorldCharacter _characterPrefab = null!;
    [SerializeField] WorldMap _worldMap = null!;
    [SerializeField] WorldCamera _worldCamera = null!;
    
    WorldCharacter? _hero;
    bool _initialized;

    public WorldCharacter? Hero => _hero;

    public void InitializeIfNeeded()
    {
      if (_initialized)
      {
        return;
      }
      
      _hero = ComponentUtils.Instantiate(_characterPrefab);
      var storedPosition = PositionStorage.GetPosition(_registry);
      _hero.transform.position = storedPosition == null ?
        Vector3.zero :
        _worldMap.ToWorldPosition(storedPosition.Value);
      _worldCamera.transform.position = new Vector3(
        _hero.transform.position.x,
        _hero.transform.position.y,
        _worldCamera.transform.position.z);
      _hero.Initialize(_worldMap);
      _initialized = true;
    }

    public void MoveHero(List<Vector3> path, Action? onArrive = null)
    {
      Errors.CheckNotNull(_hero).MoveOnPath(path, onArrive);
    }

    public Vector3Int CurrentHeroPosition()
    {
      var result = _worldMap.FromWorldPosition(Errors.CheckNotNull(_hero).transform.position);
      return new Vector3Int(result.X, result.Y, 0);
    }

    public void CreateOrUpdateCharacter(MapPosition tilePosition, WorldMapCharacter tileCharacter)
    {
      var character = ComponentUtils.Instantiate(_characterPrefab);
      character.transform.position = _worldMap.ToWorldPosition(tilePosition);
      character.Initialize(_worldMap);
      var preset = _registry.AssetService.GetCharacterPreset(tileCharacter.Appearance);
      if (preset != null)
      {
        character.ApplyCharacterPreset(preset);
      }
      character.SetFacingDirection(tileCharacter.FacingDirection);
    }
  }
}