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
#if UNITY_EDITOR
using UnityEditor;
using UnityEditor.Build;
#endif

namespace Riftcaller.Tools
{
  public static class ScriptingDefineSymbols
  {
    public static void Update()
    {
#if UNITY_EDITOR
      var defines = new List<string>();
      if (EditorPrefs.GetBool(UseDevelopmentServer.SettingName))
      {
        defines.Add(UseDevelopmentServer.DefineName);
      }
      
      PlayerSettings.SetScriptingDefineSymbols(NamedBuildTarget.Android, defines.ToArray());
      PlayerSettings.SetScriptingDefineSymbols(NamedBuildTarget.iOS, defines.ToArray());
      PlayerSettings.SetScriptingDefineSymbols(NamedBuildTarget.Standalone, defines.ToArray());
#endif
    }
  }
}