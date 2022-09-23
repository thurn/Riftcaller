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

using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine.UIElements;

#nullable enable

namespace Spelldawn.Masonry
{
  public static class Reconciler
  {
    /// <summary>
    /// Runs the tree diff algorithm, updating the Visual Element hierarchy to match the new node state.
    /// </summary>
    /// <para>
    /// This algorithm handles two cases: it generates a new VisualElement hierarchy from a Node, and it mutates
    /// a previously-generated VisualElement hierarchy to match a new Node.
    /// </para>
    /// <param name="registry">Service registry for asset fetching during rendering</param>
    /// <param name="node">The node to render</param>
    /// <param name="previousElement">Optionally, a previously-rendered VisualElement which should be updated to match
    /// the new Node state</param>
    /// <returns>Either a new VisualElement matching the provided node, or null if <paramref name="previousElement"/>
    /// was mutated to match the provided node instead.</returns>
    public static VisualElement? Update(
      Registry registry,
      Node node,
      VisualElement? previousElement = null)
    {
      var nodeType = node.NodeType?.NodeTypeCase ?? NodeType.NodeTypeOneofCase.None;
      if (previousElement != null &&
          ((IMasonElement)previousElement).NodeType == nodeType)
      {
        // If node types match, reuse this node
        return UpdateWhenMatching(registry, node, previousElement);
      }
      else
      {
        // Otherwise, create a new VisualElement matching this node
        return UpdateWhenNew(registry, node);
      }
    }

    static VisualElement? UpdateWhenMatching(
      Registry registry,
      Node node,
      VisualElement previousElement)
    {
      UpdateChildren(registry, node, previousElement, previousElement);
      Mason.ApplyToElement(registry, previousElement, node);
      return null;
    }

    static VisualElement UpdateWhenNew(Registry registry, Node node)
    {
      var result = Mason.CreateElement(node);
      UpdateChildren(registry, node, result);
      Mason.ApplyToElement(registry, result, node);
      return result;
    }

    static void UpdateChildren(Registry registry,
      Node node,
      VisualElement addTo,
      VisualElement? previousElement = null)
    {
      var count = 0;
      while (count < node.Children.Count)
      {
        var child = node.Children[count];
        if (previousElement != null && count < previousElement.childCount)
        {
          // Element exists in previous tree.
          Update(
            registry,
            child,
            previousElement[count]);
        }
        else
        {
          addTo.Add(UpdateWhenNew(registry, child));
        }

        count++;
      }

      if (previousElement != null)
      {
        while (count < previousElement.childCount)
        {
          previousElement.RemoveAt(count);
          count++;
        }
      }
    }
  }
}