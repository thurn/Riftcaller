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
using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine;
using UnityEngine.Tilemaps;

namespace Spelldawn.World
{
  public sealed class WorldMap : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] Tilemap _worldTilemap = null!;

    public IEnumerator HandleUpdateWorldMap(UpdateWorldMapCommand command)
    {
      foreach (var tile in command.Tiles)
      {
        var instance = ScriptableObject.CreateInstance<Tile>();
        instance.sprite = _registry.AssetService.GetSprite(tile.SpriteAddress);
        _worldTilemap.SetTile(new Vector3Int(tile.X, tile.Y, tile.Z), instance);
      }
      yield break;
    }
  }
}