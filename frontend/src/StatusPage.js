import React from 'react';
import SyntaxHighlighter from 'react-syntax-highlighter';
import { atomOneLight } from 'react-syntax-highlighter/dist/esm/styles/hljs';
import { Button, PageHeader } from 'antd';

import './stylesheets/StatusPage.css'

const axios = require('axios');

class StatusPage extends React.Component {
    constructor(props) {
        super(props);
    }
    render() {
        var jobId = this.props.jobId || window.localStorage.getItem("jobId");
        var view = null;
        if (jobId && jobId != "null") {
            view = <StatusDetailsPage 
                handleJobIdChange={this.props.handleJobIdChange}
                jobId={jobId}
                />;
        } else {
            view = <StatusListPage
                handleJobIdChange={this.props.handleJobIdChange}
            />;
        }
        return <div className='StatusPage'>
            {view}
        </div>
    }
}

class StatusDetailsPage extends React.Component {
	constructor(props) {
		super(props);
		this.state = {
			data: {
				"id": null,
				"created_time": null,
				"updated_time": null,
				"submission": {
					"source_code": null,
					"language": null,
					"user_id": null,
					"contest_id": null,
					"problem_id": null 
				},
				"state": null,
				"result": null,
				"score": null,
				"cases": [ ]
			}
		}
        this.handleShowList = this.handleShowList.bind(this);
	}
	componentDidMount() {
		this.getApiData();
	}

	getApiData() {
        if (this.props.jobId) {
            axios.get("/jobs/" + this.props.jobId).then((res) => {
                if (res && res.status === 200) {
                    this.setState({
                        data: res.data
                    });
                } else {
                    console.log("Error");
                }
            }).catch(err => {
                console.log(err);
            });
        }
	}

    handleShowList() {
        this.props.handleJobIdChange(null);
    }

	render() {
		var detailList = [];
		this.state.data.cases.forEach((item) => {
			detailList.push(<div key={item.id} className='detailItem'>
				<div>编号：{item.id}</div>
				<div>结果：{item.result}</div>
				<div>耗时：{item.time}</div>
				<div>内存：{item.memory}</div>
				<div>信息：{item.info}</div>
			</div>);
		});
		return <div className='StatusDetailsPage'>
            <PageHeader 
                title="评测结果">
                <div id='info'>
                    <div className='line'>
                        <p>评测时间：{this.state.data.created_time}</p>
                        <p>评测编号：{this.state.data.id}</p>
                        <p>评测得分：{this.state.data.score}</p>
                    </div>
                    <div className='line'>
                        <p>比赛编号：{this.state.data.submission.contest_id}</p>
                        <p>题目编号：{this.state.data.submission.problem_id}</p>
                        <p>用户编号：{this.state.data.submission.user_id}</p>
                    </div>
                    <div className='line'>
                        <p>评测语言：{this.state.data.submission.language}</p>
                        <p>评测状态：{this.state.data.state}</p>
                        <p>评测结果：{this.state.data.result}</p>
                    </div>
                </div>
            </PageHeader>

            <div>
                <p>源代码：</p>
                <div id="source">
                    <SyntaxHighlighter language={this.state.data.submission.language}
                        style={atomOneLight}
                        showLineNumbers={true}
                        wrapLines={true}>
                        {this.state.data.submission.source_code}
                    </SyntaxHighlighter>
                </div>
            </div>
			<div id='detail'>
				{detailList}
			</div>
            <div id="show">
                <Button onClick={this.handleShowList}>评测列表</Button>
            </div>
		</div>
	}
}

class StatusListPage extends React.Component {
    constructor(props) {
        super(props);
        this.state = {
            data: [
            ]
        };
        this.handleShowDetail = this.handleShowDetail.bind(this);
    }
    componentDidMount() {
        this.getApiData();
    }

    getApiData() {
        axios.get("/jobs").then((res) => {
            if (res && res.status === 200) {
                this.setState({
                    data: res.data
                });
            } else {
                console.log("Error");
            }
        }).catch(err => {
            console.log(err);
        });
    }

    handleShowDetail(jobId) {
        this.props.handleJobIdChange(jobId);
    }

    render() {
        var view = [];
        this.state.data.reverse().forEach((item) => {
            view.push(<div key={item.id} 
                onClick={(e) => this.handleShowDetail(item.id)}
                className='StatusItem'>
                <p>评测时间：{item.created_time}</p>
                <p>评测编号：{item.id}</p>
                <p>评测得分：{item.score}</p>
                <p>比赛编号：{item.submission.contest_id}</p>
                <p>题目编号：{item.submission.problem_id}</p>
                <p>用户编号：{item.submission.user_id}</p>
                <p>评测语言：{item.submission.language}</p>
                <p>评测状态：{item.state}</p>
                <p>评测结果：{item.result}</p>
            </div>)
        });
        return <div className='StatusListPage'>
            {view}
        </div>
    }
}

export default StatusPage;