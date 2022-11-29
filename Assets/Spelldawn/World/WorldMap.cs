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
        _worldTilemap.SetTile(new Vector3Int(tile.Position.X, tile.Position.Y, tile.ZIndex), instance);
      }
      yield break;
    }

    public Vector3 ToTilePosition(WorldPosition worldPosition) =>
      _worldTilemap.layoutGrid.CellToWorld(new Vector3Int(worldPosition.X, worldPosition.Y, 0));

    /// <summary>
    /// Character positions are offset from actual world positions in order to produce the correct sprite ordering.
    /// </summary>
    public Vector3 ToCharacterPosition(WorldPosition worldPosition)
    {
      var result = ToTilePosition(worldPosition);
      result.y -= 2.25f;
      return result;
    }
  }
}