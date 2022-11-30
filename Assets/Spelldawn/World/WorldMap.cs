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
using System.Collections.Generic;
using System.Linq;
using DG.Tweening;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.Tilemaps;

namespace Spelldawn.World
{
  public sealed class WorldMap : MonoBehaviour, Dijkstra<Vector3Int>.IGraph
  {
    static readonly Vector3Int Left = new(-1, 0, 0);
    static readonly Vector3Int Right = new(1, 0, 0);
    static readonly Vector3Int Down = new(0, -1, 0);
    static readonly Vector3Int Downleft = new(-1, -1, 0);
    static readonly Vector3Int Downright = new(1, -1, 0);
    static readonly Vector3Int Up = new(0, 1, 0);
    static readonly Vector3Int Upleft = new(-1, 1, 0);
    static readonly Vector3Int Upright = new(1, 1, 0);
    static readonly Vector3Int[] DirectionsWhenYIsEven = { Left, Right, Down, Downleft, Up, Upleft };
    static readonly Vector3Int[] DirectionsWhenYIsOdd = { Left, Right, Down, Downright, Up, Upright };
    
    [SerializeField] Registry _registry = null!;
    [SerializeField] Tilemap _worldTilemap = null!;
    [SerializeField] GameObject _selectedHex = null!;
    readonly HashSet<WorldPosition> _walkableTiles = new();

    public IEnumerator HandleUpdateWorldMap(UpdateWorldMapCommand command)
    {
      _walkableTiles.Clear();
      foreach (var tile in command.Tiles)
      {
        var instance = ScriptableObject.CreateInstance<Tile>();
        instance.sprite = _registry.AssetService.GetSprite(tile.SpriteAddress);
        _worldTilemap.SetTile(new Vector3Int(tile.Position.X, tile.Position.Y, tile.ZIndex), instance);
        
        if (tile.Walkable && tile.ZIndex == 0)
        {
          _walkableTiles.Add(tile.Position);
        }
      }

      yield break;
    }

    public Vector3 ToTilePosition(WorldPosition worldPosition) =>
      _worldTilemap.layoutGrid.CellToWorld(new Vector3Int(worldPosition.X, worldPosition.Y, 0));

    public WorldPosition FromTilePosition(Vector3 tilePosition)
    {
      var position = _worldTilemap.layoutGrid.WorldToCell(tilePosition);
      return new WorldPosition
      {
        X = position.x,
        Y = position.y
      };
    }

    /// <summary>
    /// Character positions are offset from actual world positions in order to produce the correct sprite ordering.
    /// </summary>
    public Vector2 ToCharacterPosition(WorldPosition worldPosition)
    {
      var result = ToTilePosition(worldPosition);
      result.y -= 2.25f;
      return result;
    }
    
    public WorldPosition FromCharacterPosition(Vector2 characterPosition)
    {
      characterPosition.y += 2.25f;
      var result = FromTilePosition(characterPosition);
      return result;
    }    

    public void OnClick(Vector3 position)
    {
      var cellPosition = _worldTilemap.layoutGrid.WorldToCell(new Vector3(position.x, position.y, 0));
      
      if (_walkableTiles.Contains(ToWorldPosition(cellPosition)))
      {
        var pos = _worldTilemap.CellToWorld(cellPosition);
        var selected = ComponentUtils.InstantiateGameObject(_selectedHex);
        selected.transform.position = pos;

        TweenUtils.Sequence("HexClick")
          .Append(selected.transform.DOScale(Vector3.zero, 0.5f).SetEase(Ease.InBack))
          .AppendCallback(() => Destroy(selected));
        
        var start = _registry.CharacterService.CurrentHeroPosition();
        var path = Dijkstra<Vector3Int>.ShortestPath(this, start, cellPosition);
        _registry.CharacterService.MoveHero(path.Select(v => ToCharacterPosition(new WorldPosition
        {
          X = v.x,
          Y = v.y
        })).ToList());        
      }
    }

    public List<Vector3Int> FindNeighbors(Vector3Int vertex)
    {
      var result = new List<Vector3Int>();
      var directions = (vertex.y % 2) == 0 ? DirectionsWhenYIsEven : DirectionsWhenYIsOdd;
      
      foreach (var direction in directions)
      {
        var neighborPos = vertex + direction;
        
        if (_walkableTiles.Contains(ToWorldPosition(neighborPos)) && 
            neighborPos.x >= _worldTilemap.cellBounds.min.x && 
            neighborPos.x < _worldTilemap.cellBounds.max.x &&
            neighborPos.y >= _worldTilemap.cellBounds.min.y &&
            neighborPos.y < _worldTilemap.cellBounds.max.y)
        {
          result.Add(neighborPos);
        }
      }

      return result;
    }
    
    public List<Vector3Int> Vertices()
    {
      var result = new List<Vector3Int>();
      for (var x = _worldTilemap.cellBounds.min.x; x < _worldTilemap.cellBounds.max.x; x++)
      {
        for (var y = _worldTilemap.cellBounds.min.y; y < _worldTilemap.cellBounds.max.y; y++)
        {
          result.Add(new Vector3Int(x, y, 0));
        }
      }

      return result;
    }

    WorldPosition ToWorldPosition(Vector3Int vector) => new() { X = vector.x, Y = vector.y };
  }
}