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

using System;
using UnityEngine;
using Object = UnityEngine.Object;

#nullable enable

namespace Riftcaller.Utils
{
  public static class ComponentUtils
  {
    public static T Instantiate<T>(T prefabComponent, Transform? parent = null) where T : Component =>
      InstantiateGameObject<T>(Errors.CheckNotNull(prefabComponent).gameObject, parent);

    public static T InstantiateGameObject<T>(GameObject prefab, Transform? parent = null) where T : Component
    {
      Errors.CheckNotNull(prefab);
      var instantiated = Object.Instantiate(prefab, parent);
      var result = instantiated.GetComponent<T>();
      if (!result)
      {
        throw new NullReferenceException($"Expected a component of type {typeof(T).FullName}");
      }

      if (!parent)
        // Instantiate things safely out of view if there's no parent specified :)
      {
        instantiated.transform.position = 1000f * Vector3.one;
      }

      return result;
    }
    
    public static GameObject InstantiateGameObject(GameObject prefab, Transform? parent = null)
    {
      Errors.CheckNotNull(prefab);
      var result = Object.Instantiate(prefab, parent);

      if (!parent)
      {
        result.transform.position = 1000f * Vector3.one;
      }

      return result;
    }    

    public static T GetComponent<T>(Component component) where T : Component
    {
      Errors.CheckNotNull(component);
      var result = component.GetComponent<T>();
      if (!result)
      {
        throw new NullReferenceException(
          $"Expected a component of type {typeof(T).FullName} on {component.gameObject.name}");
      }

      return result;
    }

    public static T GetComponent<T>(GameObject gameObject) where T : Component
    {
      Errors.CheckNotNull(gameObject);
      var result = gameObject.GetComponent<T>();
      if (!result)
      {
        throw new NullReferenceException($"Expected a component of type {typeof(T).FullName} on {gameObject.name}");
      }

      return result;
    }
  }
}