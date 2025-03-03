import * as m from 'motion/react-m';
import styled from 'styled-components';

import { SB_SPACING } from '@app/theme/styles.ts';

export const SidebarNavigationWrapper = styled(m.div)`
    height: 100%;
    width: 100%;
    top: 0;
    left: 0;
    padding: 20px;
    position: absolute;
    display: flex;
    flex-shrink: 0;
    gap: ${SB_SPACING}px;
    pointer-events: all;
    z-index: 10;
`;

export const SidebarGrid = styled.div`
    width: 100%;
    height: 100%;
    position: relative;
    display: grid;
    grid-template-rows: 1fr;
    grid-template-columns: min-content auto min-content;
    justify-items: stretch;
    grid-template-areas: 'miner . wallet';
`;
export const SidebarWrapper = styled(m.div)`
    pointer-events: all;
    background: ${({ theme }) => theme.palette.background.default};
    box-shadow: 0 0 45px 0 rgba(0, 0, 0, 0.15);
    flex-direction: column;
    border-radius: 20px;
    height: 100%;

    position: relative;
    overflow: hidden;

    & * {
        pointer-events: all;
    }
`;
