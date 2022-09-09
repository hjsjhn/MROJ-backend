import React from 'react';
import Page from './Page.js'
const axios = require('axios');

class App extends React.Component {
	constructor(props) {
		super(props);
		this.state = {
		};
	}

	render() {
		return <div className="App">
			<Page />
		</div>
	}
}

export default App;
