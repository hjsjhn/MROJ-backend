import React from 'react';
import ContestRankPage from './ContestPage';

const axios = require('axios');

class HomePage extends React.Component {
	constructor(props) {
		super(props);
		this.state = {
		}
	}
	render() {
		return <div className='HomePage'>
		<ContestRankPage contestId={0}/>
		</div>
	}
}

export default HomePage;