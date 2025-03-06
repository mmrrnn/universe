import { memo, ReactNode, useEffect } from 'react';
import { IoChevronForwardOutline } from 'react-icons/io5';
import { setAnimationProperties } from '@tari-project/tari-tower';

import { setCurrentSidebar, setSidebarOpen, SidebarType, useUIStore } from '@app/store/useUIStore.ts';
import { WalletOutlineSVG } from '@app/assets/icons/wallet-outline.tsx';
import { CubeOutlineSVG } from '@app/assets/icons/cube-outline.tsx';
import { SB_MINI_WIDTH, SB_SPACING, SB_WIDTH } from '@app/theme/styles.ts';
import { HoverIconWrapper, NavIconWrapper, NavigationWrapper, StyledIconButton } from './SidebarMini.styles.ts';

interface NavButtonProps {
    children: ReactNode;
    isActive?: boolean;
    onClick?: () => void;
}
function NavButton({ children, isActive, onClick }: NavButtonProps) {
    const sidebarOpen = useUIStore((s) => s.sidebarOpen);
    const rotate = sidebarOpen ? '180deg' : '0deg';
    const activeIcon = isActive ? (
        <>
            <HoverIconWrapper
                whileHover={{ opacity: 1 }}
                animate={{ rotate, transition }}
                style={{
                    opacity: 0,
                    rotate,
                }}
            >
                <IoChevronForwardOutline size={28} />
            </HoverIconWrapper>
        </>
    ) : null;
    return (
        <StyledIconButton variant="secondary" active={isActive} onClick={onClick}>
            {activeIcon}
            <NavIconWrapper>{children}</NavIconWrapper>
        </StyledIconButton>
    );
}

const transition = { rotate: { type: 'spring' }, opacity: { delay: 0.05 } };
const Navigation = memo(function Navigation() {
    const sidebarOpen = useUIStore((s) => s.sidebarOpen);
    const currentSidebar = useUIStore((s) => s.currentSidebar);

    const miningActive = currentSidebar === 'mining';

    function handleActiveSidebar(sidebarType: SidebarType) {
        if (currentSidebar === sidebarType) {
            setSidebarOpen(!sidebarOpen);
        } else {
            setCurrentSidebar(sidebarType);
            if (!sidebarOpen) {
                setSidebarOpen(true);
            }
        }
    }
    useEffect(() => {
        const offset = (!sidebarOpen ? SB_MINI_WIDTH : SB_WIDTH) + SB_SPACING * 2;
        setAnimationProperties([
            { property: 'offsetX', value: offset },
            { property: 'cameraOffsetX', value: offset / window.innerWidth },
        ]);
    }, [sidebarOpen]);

    const miningSection = (
        <NavButton onClick={() => handleActiveSidebar('mining')} isActive={miningActive}>
            <CubeOutlineSVG />
        </NavButton>
    );
    const walletSection = (
        <NavButton onClick={() => handleActiveSidebar('wallet')} isActive={!miningActive}>
            <WalletOutlineSVG />
        </NavButton>
    );
    return (
        <NavigationWrapper>
            {miningSection}
            {walletSection}
        </NavigationWrapper>
    );
});

export default Navigation;
