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

using Riftcaller.Game;
using UnityEditor;
using UnityEngine;

#nullable enable

namespace Riftcaller.Editors
{
  [CustomEditor(typeof(ActionDisplay))]
  public sealed class ActionDisplayEditor : Editor
  {
    public override void OnInspectorGUI()
    {
      DrawDefaultInspector();

      if (GUILayout.Button("Increment"))
      {
        ((ActionDisplay)target).GainActions(1);
      }

      if (GUILayout.Button("Decrement"))
      {
        ((ActionDisplay)target).SpendActions(1);
      }
    }
  }
}