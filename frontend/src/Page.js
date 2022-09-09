import React from 'react';
import HomePage from './HomePage.js';
import SubmitPage from './SubmitPage.js';
import StatusPage from './StatusPage.js'
import ContestPage from './ContestPage.js'
import 'antd/dist/antd.css';
import './stylesheets/Page.css'

const axios = require('axios');

class Page extends React.Component {
	constructor(props) {
		super(props);

		this.state = {
			curUser: window.localStorage.getItem("curUser") || 0,
			curPage: window.localStorage.getItem("curPage") || "home"
		};
		this.handlePageChange = this.handlePageChange.bind(this);
	}

	handlePageChange(page) {
		this.setState({
			curPage: page,
		});
		window.localStorage.setItem("curPage", page);
	}

	render() {
		return <div className="Page">
			<div className='Header'>
				<PageHeader curUser={this.state.curUser}/>
			</div>
			<div className="Content">
				<PageSidebar 
					curPage={this.state.curPage}
					handlePageChange={this.handlePageChange}
				/>
				<PageBody 
					curPage={this.state.curPage} 
					curUser={this.state.curUser}
					handlePageChange={this.handlePageChange}
				/>
			</div>
		</div>
	}
}

class PageHeader extends React.Component {
	constructor(props) {
		super(props);
		this.state = {
		};

	}

	render() {
		var LogoDiv = (<div className='LogoDiv'>
			MROJ
		</div>)
		return <div className="PageHeader">
			{LogoDiv}
		</div>
	}
}

class PageSidebar extends React.Component {
	constructor(props) {
		super(props);
		this.state = {
		};
	}

	render() {
		return <div className="PageSidebar">
			<SidebarItem 
				id="home" name="首页"
				curPage={this.props.curPage} 
				handlePageChange={this.props.handlePageChange}
			/>
			<SidebarItem 
				id="submit"  name="提交代码" 
				curPage={this.props.curPage}
				handlePageChange={this.props.handlePageChange}
			/>
			<SidebarItem 
				id="contest"  name="比赛详情" 
				curPage={this.props.curPage}
				handlePageChange={this.props.handlePageChange}
			/>
			<SidebarItem 
				id="status"  name="评测信息" 
				curPage={this.props.curPage}
				handlePageChange={this.props.handlePageChange}
			/>
		</div>
	}
}

class SidebarItem extends React.Component {
	constructor(props) {
		super(props);
		this.state = {
		};
	}

	render() {
		var className = null;
		if (this.props.id === this.props.curPage)
			className = "SidebarItem active";
		else 
			className = "SidebarItem";
		return <div className={className} 
			onClick={(e) => this.props.handlePageChange(this.props.id)}>
			{this.props.name}
		</div>
	}
}

class PageBody extends React.Component {
	constructor(props) {
		super(props);
		this.state = {
			jobId: window.localStorage.getItem("jobId"),
			contestId: window.localStorage.getItem("contestId")
		};
		this.handleJobIdChange = this.handleJobIdChange.bind(this);
		this.handleContestIdChange = this.handleContestIdChange.bind(this);
	}

	handleJobIdChange(jobId) {
		this.setState({
			jobId: jobId,
		});
		window.localStorage.setItem("jobId", jobId);
	}

	handleContestIdChange(contestId) {
		this.setState({
			contestId: contestId
		});
		window.localStorage.setItem("contestId", contestId);
	}

	render() {
		var ViewList = {
			"home": <HomePage 
			/>,
			"submit": <SubmitPage 
				handlePageChange={this.props.handlePageChange}
				handleJobIdChange={this.handleJobIdChange}
				/>,
			"status": <StatusPage 
				handlePageChange={this.props.handlePageChange} 
				handleJobIdChange={this.handleJobIdChange}
				jobId={this.state.jobId}
				/>,
			"contest": <ContestPage 
				handleChangeState={this.props.handlePageChange}
				handleContestIdChange={this.handleContestIdChange}
			/>
		};
		var view = ViewList[this.props.curPage];
		return <div className="PageBody">
			{view}
		</div>
	}
}

export default Page;