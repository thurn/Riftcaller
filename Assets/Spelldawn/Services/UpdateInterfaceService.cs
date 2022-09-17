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
using System.Collections;
using System.Linq;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.UIElements;

namespace Spelldawn.Services
{
  public sealed class UpdateInterfaceService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;

    public IEnumerator HandleUpdateInterface(UpdateInterfaceElementCommand command) =>
      command.InterfaceUpdateCase switch
      {
        UpdateInterfaceElementCommand.InterfaceUpdateOneofCase.AnimateToChildIndex => HandleAnimateToChildIndex(
          command.ElementName, command.AnimateToChildIndex),
        UpdateInterfaceElementCommand.InterfaceUpdateOneofCase.AnimateToElementPosition =>
          HandleAnimateToElementPosition(command.ElementName, command.AnimateToElementPosition),
        UpdateInterfaceElementCommand.InterfaceUpdateOneofCase.Destroy => HandleDestroy(command.ElementName),
        UpdateInterfaceElementCommand.InterfaceUpdateOneofCase.UpdateText => HandleUpdateText(command.ElementName,
          command.UpdateText),
        _ => throw new ArgumentOutOfRangeException()
      };

    IEnumerator HandleUpdateText(string elementName, UpdateText command)
    {
      throw new NotImplementedException();
    }

    IEnumerator HandleDestroy(string elementName)
    {
      throw new NotImplementedException();
    }

    IEnumerator HandleAnimateToElementPosition(string elementName, AnimateToElementPosition command)
    {
      throw new NotImplementedException();
    }

    IEnumerator HandleAnimateToChildIndex(string elementName, AnimateToChildIndex command)
    {
      Debug.Log($"HandleAnimateToChildIndex");
      var element = FindElement(elementName);
      var parent = FindElement(command.ParentElementName);
      element.RemoveFromHierarchy();
      if (parent.childCount >= command.Index)
      {
        parent.Add(element);
      }
      else
      {
        parent.Insert((int)command.Index, element);
      }
      
      // Remove absolute position, if any
      element.style.position = Position.Relative;
      element.style.top = new StyleLength(StyleKeyword.Null);
      element.style.right = new StyleLength(StyleKeyword.Null);
      element.style.bottom = new StyleLength(StyleKeyword.Null);
      element.style.left = new StyleLength(StyleKeyword.Null);
      yield break;
    }

    VisualElement FindElement(string elementName)
    {
      var results = _registry.DocumentService.RootVisualElement.Query(elementName).Build();
      Errors.CheckState(results.Count() == 1, $"Expected exactly 1 {elementName} but got {results.Count()}");
      return results.First();
    }
  }
}