import { useMemo, useRef } from "react";
import MgbaWrapper, { MgbaStandalone } from "./components/mgba/mgbaWrapper";
import { MobileController } from "./components/mobileController/mobileController";
import { MgbaHandle } from "./components/mgba/mgba";
import { GbaKey } from "./components/mgba/bindings";
import styled, { createGlobalStyle } from "styled-components";

const GlobalStyle = createGlobalStyle`
    body {
        margin: 0;
    }
`;

const EmulatorWrapper = styled.div`
  height: 100vh;
`;

export function App() {
  const mgba = useRef<MgbaHandle>(null);

  const mgbaHandle = useMemo(
    () => ({
      restart: () => mgba.current?.restart(),
      buttonPress: (key: GbaKey) => mgba.current?.buttonPress(key),
      buttonRelease: (key: GbaKey) => mgba.current?.buttonRelease(key),
    }),
    []
  );

  return (
    <>
      <GlobalStyle />
      <EmulatorWrapper>
        <MgbaWrapper gameUrl="built-to-scale.gba" ref={mgba} />
        <MobileController mgba={mgbaHandle} />
      </EmulatorWrapper>
    </>
  );
}
