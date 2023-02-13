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

using UnityEditor;

namespace Spelldawn.Tools
{
  public static class UseProductionServer
  {
    const string MenuName = "Tools/Use Production Server";
    public const string SettingName = "UseProductionServer";
    public const string DefineName = "USE_PRODUCTION_SERVER";
    
#if UNITY_EDITOR 
    public static bool ShouldUseProductionServer
    {
      get => EditorPrefs.GetBool(SettingName, false);
      set => EditorPrefs.SetBool(SettingName, value);
    }

    [MenuItem(MenuName)]
    static void ToggleAction()
    {
      ShouldUseProductionServer = !ShouldUseProductionServer;
    }
  
    [MenuItem(MenuName, true)]
    static bool ToggleActionValidate()
    {
      Menu.SetChecked(MenuName, ShouldUseProductionServer);
      ScriptingDefineSymbols.Update();
      return true;
    }
    
#elif USE_PRODUCTION_SERVER
    public static bool ShouldUseProductionServer => true;
#else
    public static bool ShouldUseProductionServer => false;
#endif
  }
}