import { HotkeysEvents, HotkeyScope } from '@blueprintjs/core/lib/esm/components/hotkeys/hotkeysEvents';
import React from 'react';
import { Hotkey, Hotkeys } from '@blueprintjs/core';
import _ from 'lodash';

class GlobalHotkeys {
  private globalHotkeysEvents: HotkeysEvents;
  private hotkeys: Hotkey[];
  constructor() {
    this.globalHotkeysEvents = new HotkeysEvents(HotkeyScope.GLOBAL);
    this.hotkeys = [];
    document.addEventListener('keydown', this.globalHotkeysEvents.handleKeyDown);
    document.addEventListener('keyup', this.globalHotkeysEvents.handleKeyUp);
  }

  public register = (hotkeys: Hotkey | Hotkey[]) => {
    if (typeof hotkeys === typeof Hotkey) {
      this.registerHelper(hotkeys as Hotkey);
    } else {
      _.forEach(hotkeys, this.registerHelper);
    }
  };

  private registerHelper = (hotkey: Hotkey) => {
    const index = this.hotkeys.map(k => k.props.combo).indexOf(hotkey.props.combo);
    if (index > -1) {
      this.hotkeys.splice(index, 1);
    }
    this.hotkeys.push(hotkey);
    const hotkeysEl = (
      <Hotkeys>
        {this.hotkeys.map(h => {
          return <Hotkey {...h.props} />;
        })}
      </Hotkeys>
    );
    this.globalHotkeysEvents.setHotkeys(hotkeysEl.props);
  };
}

export const globalHotkeys = new GlobalHotkeys();
