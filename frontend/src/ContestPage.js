import React from 'react';
import { PageHeader, Row, Col, Button, Input, Table } from 'antd';
import './stylesheets/ContestPage.css';

const axios = require('axios');

class ContestPage extends React.Component {
	constructor(props) {
		super(props);
		this.state = {

		};
	}
	render() {
        var contestId = this.props.contestId || window.localStorage.getItem("contestId");
        var view = null;
		if (contestId && contestId != "null") {
            view = <ContestDetailsPage
				handleContestIdChange={this.props.handleContestIdChange}
				contestId={contestId}
                />;
        } else {
            view = <ContestListPage
				handleContestIdChange={this.props.handleContestIdChange}
            />;
        }
        return <div className='ContestPage'>
            {view}
        </div>
	}
}

class ContestListPage extends React.Component {
	constructor(props) {
		super(props);
		this.state = {
			form: {
				"id": null,
				"name": "Rust Course Project 2",
				"from": "2022-08-27T02:05:29.000Z",
				"to": "2022-08-27T02:05:30.000Z",
				"problem_ids": [
					0, 1
				],
				"user_ids": [
					0, 1, 2
				],
				"submission_limit": 32
			},
			data: []
		};
		this.handleChangeState = this.handleChangeState.bind(this);
		this.createNewContest = this.createNewContest.bind(this);
		this.handleShowDetail = this.handleShowDetail.bind(this);
	}

	componentDidMount() {
		this.getApiData();
	}

	getApiData() {
		axios.get("/contests").then((res) => {
			if (res && res.status === 200) {
				console.log(res.data);
				this.setState({
					data: res.data
				});
			} else {
				console.log("Error");
			}
		}).catch((err) => {
			console.log(err);
		});
	}

	handleChangeState(e) {
		var type = e.target.dataset.type;
		var value = e.target.value;
		var oldForm = this.state.form;
		oldForm[type] = value;
		this.setState({
			form: oldForm
		});
	}

	createNewContest() {
		var data = this.state.form;
		axios.post("/contests", data).then((res) => {
			if (res && res.status === 200) {
				this.props.handleContestIdChange(res.data.id)
			} else {
				console.log("error");
			}
		}).catch(err => {
			console.log(err);
		});
	}

	handleShowDetail(contestId) {
		console.log(contestId);
		this.props.handleContestIdChange(contestId);
	}

	render() {
		var view = [];
		
		view.push(<div key="0" className='ContestItem ContestNewItem'>
			<div>
				<span>比赛名称：</span>
				<Input type="text" data-type="name" onChange={this.handleChangeState}/>
			</div>
			<div>
				<span>开始时间：</span>
				<Input type="text" data-type="from" onChange={this.handleChangeState}/>
			</div>
			<div>
				<span>结束时间：</span>
				<Input type="text" data-type="to" onChange={this.handleChangeState}/>
			</div>
			<div>
				<Button onClick={this.createNewContest}>新建比赛</Button>
			</div>
		</div>);

		this.state.data.reverse().forEach((item) => {
			view.push(<div key={item.id} 
				className='ContestItem'
				onClick={(e) => this.handleShowDetail(item.id)}
				>
				<p>比赛名称：{item.name}</p>
				<p>比赛编号：{item.id}</p>
				<p>开始时间：{item.from}</p>
				<p>结束时间：{item.to}</p>
			</div>);
		});

		return <div className='ContestListPage'>
			<div id="head">
				<h1>比赛列表</h1>
			</div>
			<div id="body">
				{view}
			</div>
		</div>
	}
}

class ContestDetailsPage extends React.Component {
	constructor(props) {
		super(props);
		this.state = {
			ranklist: false,
			data: {
				"id": null,
				"name": null,
				"from": null,
				"to": null,
				"problem_ids": [],
				"user_ids": [],
				"submission_limit": null
			  }
		}
		this.handleShowList = this.handleShowList.bind(this);
		this.handleShowRank = this.handleShowRank.bind(this);
	}
	handleShowList() {
		this.props.handleContestIdChange(null);
	}
	handleShowRank() {
	}

	getApiData() {
		axios.get("/contests/" + this.props.contestId).then(res => {
			if (res && res.status === 200) {
				this.setState({
					data: res.data
				});
			} else {
				console.log(res);
			}
		}).catch(err => {
			console.log(err);
		});
	}

	componentDidMount() {
		this.getApiData();
	}

	render() {
		var view = <ContestRankPage contestId={this.props.contestId} contestData={this.state.data}/>
		return <div className='ContestDetailPage'>
			{view}
			<Button
				onClick={this.handleShowList}>
				比赛列表
			</Button>
		</div>
	}
	
}

class ContestRankPage extends React.Component {
	constructor(props) {
		super(props);
		this.state = {
			data: []
		};
		this.getApiData = this.getApiData.bind(this);
	}

	componentDidMount() {
		this.getApiData();
	}

	getApiData() {
		axios.get("/contests/" + this.props.contestId + "/ranklist").then(res => {
			if (res && res.status == 200) {
				this.setState({
					data: res.data
				});
			}  else {
				console.log("err");
			}
		}).catch(err => {
			console.log(err);
		});
	}

	render() {
		var view = [];

		var columns = [{
			title: '排名',
			dataIndex: 'rank',
			key: 'rank',
			align: "center",
			width: 100
		}, {
			title: '名称',
			dataIndex: 'name',
			key: 'name',
			align: "center",
			render: text => <a>{text}</a>,
		}];

		this.props.contestData.problem_ids.forEach(item => {
			columns.push({
				title: item,
				dataIndex: item,
				key: item,
				align: "center",
				width: 100
			})
		});

		var cnt = 0;
		var data = [];
		this.state.data.forEach(item => {
			var tdata = {
				key: ++cnt,
				name: item.user.name,
				rank: item.rank,
			}
			for (var i = 0; i < item.scores.length; ++i) {
				tdata[this.props.contestData.problem_ids[i]] = item.scores[i];
			}
			data.push(tdata);
		})

		return <div className='ContestRankPage'>
			<PageHeader title='排行榜' /> 
			<Row gutter={{ xs: 8, sm: 16, md: 24, lg: 32 }}>
				<Col className='gutter-row' span={16}>
					<Table columns={columns} dataSource={data} />
				</Col>
			</Row>
		</div>
	}
}

export default ContestPage;