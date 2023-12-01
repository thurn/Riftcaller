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

using UnityEditor;

namespace Riftcaller.Tools
{
  public static class IntroduceNetworkDelay
  {
    const string MenuName = "Tools/Long Network Delay %&n";
    const string SettingName = "IntroduceNetworkDelay";
    
#if UNITY_EDITOR 
    public static bool ShouldIntroduceLongNetworkDelay
    {
      get => EditorPrefs.GetBool(SettingName, false);
      set => EditorPrefs.SetBool(SettingName, value);
    }

    [MenuItem(MenuName)]
    static void ToggleAction()
    {
      ShouldIntroduceLongNetworkDelay = !ShouldIntroduceLongNetworkDelay;
    }
  
    [MenuItem(MenuName, true)]
    static bool ToggleActionValidate()
    {
      Menu.SetChecked(MenuName, ShouldIntroduceLongNetworkDelay);
      return true;
    }
#else
    public static bool ShouldIntroduceLongNetworkDelay => false;
#endif
  }
}