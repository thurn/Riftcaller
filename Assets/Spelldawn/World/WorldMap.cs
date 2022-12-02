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
    [SerializeField] Tilemap _tilemapPrefab = null!;
    [SerializeField] GameObject _selectedHex = null!;
    [SerializeField] Sprite _tmpBackground = null!;
    [SerializeField] Sprite _tmpIcon = null!;
    readonly Dictionary<(int, int), Tilemap> _tilemaps = new();
    readonly HashSet<MapPosition> _walkableTiles = new();

    void Start()
    {
      // Always create at least one tilemap to use for resolving positions
      GetTilemap(new MapPosition { X = 0, Y = 0}, 0);
      
      var instance = ScriptableObject.CreateInstance<Tile>();
      instance.sprite = _tmpBackground;
      instance.color = Color.black;
      instance.transform = Matrix4x4.Scale(new Vector3(0.6f, 0.6f, 1));
      var position = new MapPosition { X = 0, Y = -1 };
      var tilemap = GetTilemap(position, 2);
      tilemap.tileAnchor = Vector3.zero;
      tilemap.SetTile(ToVector3Int(position, 2), instance);      
      
      var i2 = ScriptableObject.CreateInstance<Tile>();
      i2.sprite = _tmpIcon;
      i2.transform = Matrix4x4.Scale(new Vector3(0.6f, 0.6f, 1));
      var p2 = new MapPosition { X = 0, Y = -1 };
      var t2 = GetTilemap(p2, 3);
      t2.tileAnchor = Vector3.zero;
      t2.SetTile(ToVector3Int(p2, 3), i2);
    }

    public IEnumerator HandleUpdateWorldMap(UpdateWorldMapCommand command)
    {
      _walkableTiles.Clear();
      foreach (var tile in command.Tiles)
      {
        // Unity has 3 different ways of managing tile sort order (global sort axis, tilemap z position,
        // and sprite sorting order). In my experience these layers are generally buggy and unreliable,
        // especially tilemap z-index. To make sure everything works as expected, we *only* rely on 
        // sprite sorting order and instantiate numerous different Tilemaps with different sorting
        // behavior.
        
        var instance = ScriptableObject.CreateInstance<Tile>();
        instance.sprite = _registry.AssetService.GetSprite(tile.SpriteAddress);
        GetTilemap(tile.Position, tile.ZIndex).SetTile(new Vector3Int(tile.Position.X, tile.Position.Y, 0), instance);

        if (tile.Walkable && tile.ZIndex == 0)
        {
          _walkableTiles.Add(tile.Position);
        }
      }

      yield break;
    }

    public void OnClick(Vector3 position)
    {
      var cellPosition = _tilemaps[(0,0)].layoutGrid.WorldToCell(new Vector3(position.x, position.y, 0));

      if (_walkableTiles.Contains(FromVector3Int(cellPosition)))
      {
        var pos = _tilemaps[(0,0)].CellToWorld(cellPosition);
        var selected = ComponentUtils.InstantiateGameObject(_selectedHex);
        selected.transform.position = pos;
        selected.GetComponent<SpriteRenderer>().sortingOrder = 
          SortOrderForTileAndZIndex(FromVector3Int(cellPosition), 5);
        
        TweenUtils.Sequence("HexClick")
          .Append(selected.transform.DOScale(Vector3.zero, 0.5f).SetEase(Ease.InExpo))
          .AppendCallback(() => Destroy(selected));

        var start = _registry.CharacterService.CurrentHeroPosition();
        var path = Dijkstra<Vector3Int>.ShortestPath(this, start, cellPosition);
        _registry.CharacterService.MoveHero(path.Select(v => ToWorldPosition(new MapPosition
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

        if (_walkableTiles.Contains(FromVector3Int(neighborPos)))
        {
          result.Add(neighborPos);
        }
      }

      return result;
    }

    public List<Vector3Int> Vertices() => _walkableTiles.Select(t => ToVector3Int(t)).ToList();

    MapPosition FromVector3Int(Vector3Int vector) => new() { X = vector.x, Y = vector.y };
    
    Vector3Int ToVector3Int(MapPosition position, int z = 0) => new() { x = position.X, y = position.Y, z = z };
    
    public Vector3 ToWorldPosition(MapPosition mapPosition) =>
      GetTilemap(mapPosition, 0).layoutGrid.CellToWorld(new Vector3Int(mapPosition.X, mapPosition.Y, 0));

    public MapPosition FromWorldPosition(Vector3 worldPosition)
    {
      var position = _tilemaps[(0, 0)].layoutGrid.WorldToCell(worldPosition);
      return new MapPosition
      {
        X = position.x,
        Y = position.y
      };
    }

    public int SortOrderForTileAndZIndex(MapPosition position, int zIndex) => (position.Y * -100) + zIndex;

    Tilemap GetTilemap(MapPosition position, int zIndex)
    {
      if (!_tilemaps.ContainsKey((position.Y, zIndex)))
      {
        var tilemap = ComponentUtils.Instantiate(_tilemapPrefab);
        tilemap.transform.SetParent(transform);
        tilemap.transform.position = Vector3.zero;
        ComponentUtils.GetComponent<TilemapRenderer>(tilemap).sortingOrder = SortOrderForTileAndZIndex(position, zIndex);
        _tilemaps[(position.Y, zIndex)] = tilemap;
        return tilemap;
      }
      else
      {
        return _tilemaps[(position.Y, zIndex)];
      }
    }
  }
}