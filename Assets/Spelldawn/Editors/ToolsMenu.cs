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

using System.Linq;
using UnityEditor;

namespace Spelldawn.Editors
{
  public static class ToolsMenu
  {
    const string MenuName = "Tools/Use Plugin";
    const string SettingName = "UsePlugin";
    
    public static bool UsePlugin
    {
      get => EditorPrefs.GetBool(SettingName, false);
      set
      {
        if (value)
        {
          AddDefineSymbols();
        }
        else
        {
          RemoveDefineSymbols();
        }
        EditorPrefs.SetBool(SettingName, value);
      }
    }

    [MenuItem(MenuName)]
    static void ToggleAction()
    {
      UsePlugin = !UsePlugin;
    }
  
    [MenuItem(MenuName, true)]
    static bool ToggleActionValidate()
    {
      Menu.SetChecked(MenuName, UsePlugin);
      return true;
    }
    
    static readonly string [] Symbols = {
      "USE_UNITY_PLUGIN"
    };
    
    static void AddDefineSymbols()
    {
      var definesString = PlayerSettings.GetScriptingDefineSymbolsForGroup
        (EditorUserBuildSettings.selectedBuildTargetGroup);
      var allDefines = definesString.Split(';').ToList();
      allDefines.AddRange(Symbols.Except(allDefines));
      PlayerSettings.SetScriptingDefineSymbolsForGroup(
        EditorUserBuildSettings.selectedBuildTargetGroup,
        string.Join(";", allDefines.ToArray()));
    }
    
    static void RemoveDefineSymbols()
    {
      var definesString = PlayerSettings.GetScriptingDefineSymbolsForGroup(
        EditorUserBuildSettings.selectedBuildTargetGroup);
      var allDefines = definesString.Split(';').ToList();
      allDefines.RemoveAll(s => Symbols.Contains(s));
      PlayerSettings.SetScriptingDefineSymbolsForGroup(
        EditorUserBuildSettings.selectedBuildTargetGroup,
        string.Join (";", allDefines.ToArray()));
    }      
  }
}