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

using System.Collections.Generic;
using System.Linq;

namespace Riftcaller.World
{
  public static class Dijkstra<TVertex> where TVertex: struct
  {
    public interface IGraph
    {
      List<TVertex> Vertices();
      
      List<TVertex> FindNeighbors(TVertex vertex);

      float GetDistance(TVertex source, TVertex destination) => 1.0f;
    }
    
    /// <summary>Returns the shortest path between 'source' and 'destination'.</summary>
    public static List<TVertex> ShortestPath(IGraph graph, TVertex source, TVertex destination)
    {
      ShortestPaths(graph, source, out _, out var prev);
      return PathToSource(destination, prev);
    }
    
    /// <summary>Returns the shortest path to the vertex in 'destinations' which is closest to 'source'.</summary>
    public static List<TVertex> ShortestPathOfDestinations(IGraph graph, TVertex source, List<TVertex> destinations)
    {
      ShortestPaths(graph, source, out var distances, out var prev);
      var closest = distances.Where(d => destinations.Contains(d.Key)).OrderBy(d => d.Value).First();
      var path = PathToSource(closest.Key, prev);
      return path;
    }

    static void ShortestPaths(
      IGraph graph, 
      TVertex source, 
      out Dictionary<TVertex, float> distances, 
      out Dictionary<TVertex, TVertex> previous)
    {
      // Code adapted from https://en.wikipedia.org/wiki/Dijkstra's_algorithm#Pseudocode

      var dist = new Dictionary<TVertex, float>();
      var prev = new Dictionary<TVertex, TVertex>();
      var q = new HashSet<TVertex>();

      foreach (var vertex in graph.Vertices())
      {
        dist[vertex] = float.PositiveInfinity;
        q.Add(vertex);
      }

      dist[source] = 0f;

      while (q.Count > 0)
      {
        var u = q.OrderBy(v => dist[v]).First();
        q.Remove(u);

        foreach (var vertex in graph.FindNeighbors(u).Where(v => q.Contains(v)))
        {
          var alt = dist[u] + graph.GetDistance(u, vertex);
          if (alt < dist[vertex])
          {
            dist[vertex] = alt;
            prev[vertex] = u;
          }
        }
      }

      distances = dist;
      previous = prev;
    }
    
    static List<TVertex> PathToSource(TVertex destination, Dictionary<TVertex, TVertex> prev)
    {
      // Read the shortest path back to 'source'
      var position = destination;
      var path = new List<TVertex>();
      while (prev.ContainsKey(position))
      {
        path.Insert(0, position);
        position = prev[position];
      }

      return path;
    }
  }
}