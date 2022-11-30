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

using System.Collections.Generic;
using System.Linq;

namespace Spelldawn.World
{
  public static class Dijkstra<TVertex> where TVertex: struct
  {
    public interface IGraph
    {
      List<TVertex> Vertices();
      
      List<TVertex> FindNeighbors(TVertex vertex);

      float GetDistance(TVertex source, TVertex destination) => 1.0f;
    }
    
    public static List<TVertex> ShortestPath(IGraph graph, TVertex source, TVertex destination)
    {
      // Code adapted from https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm
      
      var distance = new Dictionary<TVertex, float>();
      var previous = new Dictionary<TVertex, TVertex>();
      var remaining = new HashSet<TVertex>();
      
      foreach (var vertex in graph.Vertices())
      {
        distance[vertex] = float.PositiveInfinity;
        remaining.Add(vertex);
      }
      distance[source] = 0f;

      while (remaining.Count > 0)
      {
        var subject = remaining.OrderBy(v => distance[v]).First();

        if (subject.Equals(destination))
        {
          // Read the shortest path back to 'source' and then terminate
          var path = new List<TVertex>();
          while (previous.ContainsKey(subject))
          {
            path.Insert(0, subject);
            subject = previous[subject];
          }

          return path;
        }
        
        remaining.Remove(subject);

        foreach (var vertex in graph.FindNeighbors(subject))
        {
          var d = distance[subject] + graph.GetDistance(subject, vertex);
          if (d < distance[vertex])
          {
            distance[vertex] = d;
            previous[vertex] = subject;
          }
        }
      }

      return new List<TVertex>();
    }    
  }
}