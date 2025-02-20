/* eslint-disable i18next/no-literal-string */
import { Button, ButtonGroup, CategoryLabel } from '../styles';
import { relaunch } from '@tauri-apps/plugin-process';

export function EventsGroup() {
    return (
        <>
            <CategoryLabel>Events</CategoryLabel>
            <ButtonGroup>
                <Button onClick={relaunch}>Relaunch</Button>
            </ButtonGroup>
        </>
    );
}
