import styled from 'styled-components';

export const SidebarGrid = styled.div`
    gap: 8px;
    display: grid;
    height: 100%;
    place-items: center stretch;
    align-content: space-between;
    padding: 16px 10px;
    grid-template-columns: 1fr;
    grid-template-rows: auto [row2-end row4-start] auto;
    grid-template-areas:
        'top top top'
        '. . .'
        'bottom bottom bottom ';
`;

export const GridAreaTop = styled.div`
    grid-area: top;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
`;

export const GridAreaBottom = styled.div`
    display: flex;
    grid-area: bottom;
    flex-direction: column;
    position: relative;
    gap: 6px;
`;

export const RewardWrapper = styled.div`
    display: flex;
    flex-direction: column;
    height: min(400px, 30vh);
    border-radius: 10px;
    gap: 10px;
    padding: 10px 0 0 0;
    overflow: hidden;
`;
