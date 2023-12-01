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
  public static class LogRpcTime
  {
    const string MenuName = "Tools/Log RPC Time";
    const string SettingName = "LogRpcTime";
    
#if UNITY_EDITOR 
    public static bool ShouldLogRpcTime
    {
      get => EditorPrefs.GetBool(SettingName, false);
      set => EditorPrefs.SetBool(SettingName, value);
    }

    [MenuItem(MenuName)]
    static void ToggleAction()
    {
      ShouldLogRpcTime = !ShouldLogRpcTime;
    }
  
    [MenuItem(MenuName, true)]
    static bool ToggleActionValidate()
    {
      Menu.SetChecked(MenuName, ShouldLogRpcTime);
      return true;
    }
#else
    public static bool ShouldLogRpcTime => false;
#endif
  }
}