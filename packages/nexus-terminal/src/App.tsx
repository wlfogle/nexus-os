// NexusTerminal — entry component.
// Provider wraps everything so WarpStyleTerminal (and its children) can access redux.
import { Provider } from 'react-redux';
import { store } from './store';
import { WarpStyleTerminal } from './components/terminal/WarpStyleTerminal';

function App() {
  return (
    <Provider store={store}>
      {/* WarpStyleTerminal is the full-screen Warp-clone layout. */}
      <WarpStyleTerminal className="h-screen" />
    </Provider>
  );
}

export default App;
