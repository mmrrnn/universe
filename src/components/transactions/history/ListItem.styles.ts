import styled from 'styled-components';
import * as m from 'motion/react-m';
import { Typography } from '@app/components/elements/Typography.tsx';
import { convertHexToRGBA } from '@app/utils';

export const ItemWrapper = styled(m.div)`
    display: flex;
    align-items: center;
    width: 100%;
    border-radius: 10px;
    padding: 6px 0;
    flex-direction: column;
    overflow: hidden;
    position: relative;
    box-shadow: ${({ theme }) => `${convertHexToRGBA(theme.palette.contrast, 0.025)} 0 1px 2px -1px`};
    background-color: ${({ theme }) => (theme.mode === 'dark' ? '#1B1B1B' : theme.palette.background.paper)};
`;

export const HoverWrapper = styled(m.div)`
    position: absolute;
    inset: 0;
    z-index: 4;
    transition: background-color 1s ease;
    height: 100%;

    background: linear-gradient(
        90deg,
        transparent 0%,
        transparent 32%,
        ${({ theme }) => (theme.mode === 'dark' ? '#1B1B1B' : theme.palette.background.paper)} 38%,
        ${({ theme }) => (theme.mode === 'dark' ? '#1B1B1B' : theme.palette.background.paper)} 100%
    );
`;

export const ContentWrapper = styled.div`
    width: 100%;
    padding: 0 12px;
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
    height: 100%;
`;

export const Content = styled.div`
    display: flex;
    gap: 4px;
    flex-direction: row;
    align-items: center;
    height: 100%;
`;

export const BlockInfoWrapper = styled.div`
    display: flex;
    flex-direction: column;
`;
export const TitleWrapper = styled(Typography)`
    display: flex;
    color: ${({ theme }) => theme.palette.text.primary};

    font-size: 12px;
    font-weight: 500;
    line-height: 1.3;
    letter-spacing: -0.24px;
`;
export const TimeWrapper = styled(Typography)`
    display: flex;

    font-size: 11px;
    color: ${({ theme }) => theme.palette.text.secondary};
`;
export const ValueWrapper = styled.div`
    display: flex;
    gap: 3px;
    font-weight: 500;
    justify-content: flex-end;
    align-items: baseline;
`;
export const Chip = styled.div`
    display: flex;
    align-self: center;
    justify-content: center;
    align-items: center;

    text-transform: uppercase;
    border-radius: 50px;
    background-color: ${({ theme }) => theme.colors.green[700]};

    height: 14px;
    padding: 0 7px;

    color: #fff;
    text-align: center;
    font-family: Poppins, sans-serif;
    font-size: 8px;
    font-style: normal;
    font-weight: 700;
    line-height: normal;

    span {
        height: 10px;
    }
`;

export const CurrencyText = styled(Typography).attrs({ variant: 'p' })`
    display: flex;
    font-size: 11px;
    font-weight: 500;
    color: ${({ theme }) => theme.palette.text.secondary};
`;

export const ValueChangeWrapper = styled.div<{ $isPositiveValue?: boolean }>`
    display: flex;
    line-height: 11px;
    color: ${({ theme, $isPositiveValue }) =>
        $isPositiveValue ? theme.palette.success.main : theme.palette.error.main};
`;

export const ReplayButton = styled.button`
    display: flex;
    border-radius: 100%;
    position: relative;
    width: 31px;
    height: 31px;
    justify-content: center;
    background-color: ${({ theme }) => theme.palette.text.secondary};
    color: ${({ theme }) => theme.palette.text.contrast};
    box-sizing: border-box;
    transition: opacity 0.2s ease;

    &:hover {
        opacity: 0.8;
    }

    svg {
        position: relative;
        top: 50%;
        transform: translateY(-50%);
    }
`;

export const ButtonWrapper = styled(m.div)`
    position: relative;
    align-items: center;
    display: flex;
    flex-direction: row;
    padding: 0 10px;
    justify-content: flex-end;
    height: 100%;
    gap: 6px;
`;

export const FlexButton = styled.button`
    display: flex;
    height: 31px;
    padding: 8px 5px 8px 18px;
    justify-content: center;
    align-items: center;
    gap: 8px;
    border-radius: 159px;
    background:
        linear-gradient(0deg, #c9eb00 0%, #c9eb00 100%), linear-gradient(180deg, #755cff 0%, #2946d9 100%),
        linear-gradient(180deg, #ff84a4 0%, #d92958 100%);

    position: relative;
    color: ${({ theme }) => theme.colors.greyscale[950]};
    font-size: 12px;
    font-weight: 600;
    line-height: 1;
    cursor: pointer;
    box-shadow: inset 0 0 0 2px ${({ theme }) => convertHexToRGBA(theme.palette.base, 0)};
    &:hover {
        box-shadow: inset 0 0 0 2px ${({ theme }) => convertHexToRGBA(theme.palette.base, 0.4)};
    }
`;

export const GemPill = styled.div`
    border-radius: 60px;
    background: #000;
    justify-content: center;
    display: flex;
    height: 20px;
    padding: 0 5px 0 8px;
    align-items: center;
    gap: 4px;

    span {
        color: ${({ theme }) => theme.colors.greyscale[50]};
        display: flex;
        font-size: 10px;
        font-weight: 600;
        line-height: 1.1;
    }
`;

export const GemImage = styled.img`
    width: 11px;
`;

export const PlaceholderItem = styled.div<{ $isLast?: boolean }>`
    width: 100%;
    height: ${({ $isLast }) => ($isLast ? '35px' : '48px')};
    background: ${({ theme }) => (theme.mode === 'dark' ? '#222223' : '#F3F3F3')};
    border-radius: 10px;
    flex-shrink: 0;
    opacity: ${({ $isLast }) => ($isLast ? 0 : 0.75)};
`;
